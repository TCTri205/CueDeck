use std::path::Path;
use std::fs::{self, File};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, Context};
use fs2::FileExt;
use crate::db::DbManager;
use crate::cache::DocumentCache;
use tracing::{info, warn, error};

/// Name of the migration lock file
const MIGRATION_LOCK_FILE: &str = "migration.lock";

/// Name of the migration failure marker file
const MIGRATION_FAILED_MARKER: &str = "migration_failed.marker";

/// Check if migration is needed and safe to proceed
///
/// Returns true if:
/// 1. SQLite DB does not exist
/// 2. JSON cache exists
/// 3. No migration failure marker exists
/// 4. No migration lock file exists (checked securely later)
pub fn needs_migration(workspace_root: &Path) -> Result<bool> {
    let db_path = workspace_root.join(".cue").join("metadata.db");
    let json_path = workspace_root.join(".cue").join("documents.bin");
    let marker_path = workspace_root.join(".cue").join(MIGRATION_FAILED_MARKER);

    // If DB exists, we're good (unless it's corrupted, but that's handled by engine)
    if db_path.exists() {
        return Ok(false);
    }

    // If no JSON cache, nothing to migrate (fresh start)
    if !json_path.exists() {
        return Ok(false);
    }

    // If previous migration failed, don't retry automatically to avoid loops
    if marker_path.exists() {
        warn!("Previous migration failed (marker found). Skipping migration to prevent retry loop.");
        return Ok(false);
    }

    Ok(true)
}

/// Perform the migration from JSON cache to SQLite
///
/// Steps:
/// 1. Acquire exclusive lock
/// 2. Backup existing JSON cache
/// 3. Initialize SQLite DB
/// 4. Read JSON cache
/// 5. Batch insert into SQLite
/// 6. Cleanup and release lock
pub fn migrate_json_to_sqlite(workspace_root: &Path, cache: &DocumentCache) -> Result<usize> {
    let dot_cue = workspace_root.join(".cue");
    fs::create_dir_all(&dot_cue)?;

    let lock_path = dot_cue.join(MIGRATION_LOCK_FILE);
    let lock_file = File::create(&lock_path)
        .context("Failed to create migration lock file")?;

    // 1. Acquire lock (non-blocking)
    if lock_file.try_lock_exclusive().is_err() {
        warn!("Migration already in progress (lock held). Skipping.");
        return Ok(0);
    }

    info!("Starting migration from documents.bin to metadata.db...");

    // Wrap actual migration in a closure to ensure we can catch errors and cleanup
    let result = (|| -> Result<usize> {
        let db_path = dot_cue.join("metadata.db");
        // let marker_path = dot_cue.join(MIGRATION_FAILED_MARKER); // Unused variable

        // 2. Create backup of JSON cache
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let backup_path = dot_cue.join(format!("documents.bin.backup.{}", timestamp));
        
        // We persist the passed cache to a backup file if we can, or just copy the existing file
        let json_path = dot_cue.join("documents.bin");
        if json_path.exists() {
             fs::copy(&json_path, &backup_path)
                .context("Failed to backup documents.bin")?;
             info!("Created backup at {:?}", backup_path);
        }

        // 3. Initialize SQLite DB (creates tables via schema.sql)
        // If it exists (partial migration), we might be overwriting or appending.
        // For safety, if we demand a fresh migration, we might remove it first,
        // but DbManager::open handles "create if not exists".
        let mut db = DbManager::open(&db_path)?;

        // 4. Prepare batch data
        // Filter out files that no longer exist on disk to clean up the index
        let mut files_to_insert = Vec::new();
        let mut skipped_count = 0;

        for (path_buf, doc) in &cache.entries {
            // Verify file still exists on disk
            if !path_buf.exists() {
                skipped_count += 1;
                continue;
            }

            // Extract metadata from CachedDocument
            // doc.modified is SystemTime, convert to unix timestamp
            let modified_at = doc.modified
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            
            // doc.document.tokens is the token count
            let tokens = doc.document.tokens;
            
            // We need file size. Since CachedDocument doesn't store size explicitly 
            // (it might rely on fs metadata), we check the file.
            // Performance note: This adds FS IO, but migration is one-off.
            let size_bytes = match fs::metadata(path_buf) {
                Ok(m) => m.len(),
                Err(_) => 0, // Should have been caught by exists check, but just in case
            };

            files_to_insert.push((
                path_buf.clone(),
                doc.hash.clone(),
                modified_at,
                size_bytes,
                tokens
            ));
        }

        if skipped_count > 0 {
            info!("Skipping {} deleted files during migration", skipped_count);
        }

        // 5. Batch insert
        if files_to_insert.is_empty() {
            info!("No valid files to migrate.");
        } else {
            let count = db.upsert_files_batch(&files_to_insert)?;
            info!("Successfully migrated {} files/metadata to SQLite.", count);
        }

        // Success!
        Ok(files_to_insert.len())
    })();

    // Handle result: if error, create failure marker and rollback
    if let Err(e) = &result {
        error!("Migration failed: {}. Rolling back...", e);
        
        let db_path = dot_cue.join("metadata.db");
        // Attempt rollback (delete potentially corrupted DB)
        if let Err(cleanup_err) = rollback_migration(&db_path) {
            error!("Failed to clean up after migration failure: {}", cleanup_err);
        }

        // Create failure marker to prevent retry loops
        let marker_path = dot_cue.join(MIGRATION_FAILED_MARKER);
        if let Err(marker_err) = File::create(&marker_path) {
             error!("Failed to create failure marker: {}", marker_err);
        }
    }

    // Release lock
    let _ = lock_file.unlock();
    let _ = fs::remove_file(lock_path);

    result
}

/// Rollback migration by removing SQLite files
fn rollback_migration(db_path: &Path) -> Result<()> {
    if db_path.exists() {
        fs::remove_file(db_path).context("Failed to delete database file")?;
    }
    
    // Clean up WAL and SHM files if they exist
    let wal_path = db_path.with_extension("db-wal");
    if wal_path.exists() {
        fs::remove_file(wal_path).context("Failed to delete WAL file")?;
    }

    let shm_path = db_path.with_extension("db-shm");
    if shm_path.exists() {
        fs::remove_file(shm_path).context("Failed to delete SHM file")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    // use crate::cache::CachedDocument; // Unused
    // use crate::Document; // Unused

    #[test]
    fn test_needs_migration_fresh() {
        let temp = assert_fs::TempDir::new().unwrap();
        // Should be false
        assert!(!needs_migration(temp.path()).unwrap());
    }

    #[test]
    fn test_needs_migration_json_only() {
        let temp = assert_fs::TempDir::new().unwrap();
        let dot_cue = temp.child(".cue");
        dot_cue.create_dir_all().unwrap();
        dot_cue.child("documents.bin").touch().unwrap();

        assert!(needs_migration(temp.path()).unwrap());
    }

    #[test]
    fn test_needs_migration_already_migrated() {
        let temp = assert_fs::TempDir::new().unwrap();
        let dot_cue = temp.child(".cue");
        dot_cue.create_dir_all().unwrap();
        dot_cue.child("documents.bin").touch().unwrap();
        dot_cue.child("metadata.db").touch().unwrap();

        assert!(!needs_migration(temp.path()).unwrap());
    }

    #[test]
    fn test_needs_migration_prev_failure() {
        let temp = assert_fs::TempDir::new().unwrap();
        let dot_cue = temp.child(".cue");
        dot_cue.create_dir_all().unwrap();
        dot_cue.child("documents.bin").touch().unwrap();
        dot_cue.child("migration_failed.marker").touch().unwrap();

        assert!(!needs_migration(temp.path()).unwrap());
    }

    #[test]
    fn test_lock_file_prevention() {
        let temp = assert_fs::TempDir::new().unwrap();
        let dot_cue = temp.child(".cue");
        dot_cue.create_dir_all().unwrap();
        
        let lock_path = dot_cue.child("migration.lock");
        let lock_file = File::create(lock_path.path()).unwrap();
        lock_file.lock_exclusive().unwrap(); // Simulate another process

        let cache = DocumentCache::new(temp.path()).unwrap();
        let count = migrate_json_to_sqlite(temp.path(), &cache).unwrap();
        
        assert_eq!(count, 0); // Should skip
    }
}
