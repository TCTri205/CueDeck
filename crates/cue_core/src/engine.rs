use crate::cache::DocumentCache;
use crate::db::{DbManager, migration};
use crate::graph::DependencyGraph;
use cue_common::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use rayon::prelude::*;
use crate::parse_file;
use sha2::Digest;

use cue_config::Config;

/// Stateful engine for managing workspace state, caching, and graph resolution.
///
/// Designed for efficient incremental updates (watch mode).
pub struct CueEngine {
    workspace_root: PathBuf,
    cache: DocumentCache,
    graph: DependencyGraph,
    // Keep track of active keys to handle deletions effectively during full scan
    known_files: HashSet<PathBuf>,
    config: Config,
    db: Option<DbManager>,  // Optional SQLite backend (hybrid architecture)
}

impl CueEngine {
    /// Initialize a new engine
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let mut cache =
            DocumentCache::new(workspace_root).map_err(|e| std::io::Error::other(e.to_string()))?;
        cache
            .load()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        // Phase 7.3: Check and perform migration if needed
        let db = if let Ok(true) = migration::needs_migration(workspace_root) {
            tracing::info!("Migration needed. Starting migration from JSON to SQLite...");
            match migration::migrate_json_to_sqlite(workspace_root, &cache) {
                Ok(count) => {
                    tracing::info!("Migration completed successfully ({} files).", count);
                    // Open DB after successful migration
                    let db_path = workspace_root.join(".cue/metadata.db");
                    match DbManager::open(&db_path) {
                        Ok(db) => Some(db),
                        Err(e) => {
                            tracing::error!("Failed to open database after migration: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Migration failed: {}. Falling back to JSON-only mode.", e);
                    None
                }
            }
        } else {
            // Try to open existing DB if available
            let db_path = workspace_root.join(".cue/metadata.db");
            if db_path.exists() {
                 match DbManager::open(&db_path) {
                    Ok(db) => Some(db),
                    Err(e) => {
                        tracing::error!("Failed to open existing database: {}. Falling back to JSON-only.", e);
                        None
                    }
                }
            } else {
                // First run or JSON-only mode
                // If we want to start using DB from scratch for new projects, we can init it here.
                // For Phase 7.3, we enable it by default for everyone.
                let dot_cue = workspace_root.join(".cue");
                if !dot_cue.exists() {
                    std::fs::create_dir_all(&dot_cue)?;
                }
                match DbManager::open(&db_path) {
                    Ok(db) => Some(db),
                    Err(e) => {
                         tracing::warn!("Failed to initialize new database: {}", e);
                         None
                    }
                }
            }
        };

        // Load config
        let config = Config::load(workspace_root)?;

        // Initialize empty graph
        let graph = DependencyGraph::build(&[])?;

        let mut engine = Self {
            workspace_root: workspace_root.to_path_buf(),
            cache,
            graph,
            known_files: HashSet::new(),
            config,
            db,
        };

        // Initial full scan
        engine.scan_all()?;

        Ok(engine)
    }

    /// Scan all files in workspace and build initial graph
    pub fn scan_all(&mut self) -> Result<()> {
        let start_total = std::time::Instant::now();
        tracing::info!("Scanning workspace: {:?}", self.workspace_root);

        let cards_dir = self.workspace_root.join(".cuedeck/cards");
        let docs_dir = self.workspace_root.join(".cuedeck/docs");
        let mut paths = Vec::new();

        // 1. Discovery Phase
        if cards_dir.exists() {
            paths.extend(walkdir::WalkDir::new(&cards_dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "md"))
                .map(|e| e.path().to_path_buf()));
        }

        if docs_dir.exists() {
            paths.extend(walkdir::WalkDir::new(&docs_dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "md"))
                .map(|e| e.path().to_path_buf()));
        }

        // 2. Identify and Process Deletions (Serial)
        let current_files: HashSet<_> = paths.iter().cloned().collect();
        let deleted_files: Vec<_> = self.known_files
            .difference(&current_files)
            .cloned()
            .collect();

        for path in deleted_files {
            self.remove_file(&path);
        }
        self.known_files = current_files;

        // 3. Diff Analysis & Filter (Serial)
        let mut files_to_process = Vec::new();
        let mut unmodified_files = Vec::new();

        for path in paths {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    // Check if file is fresh in cache (mtime match)
                    let is_fresh = self.cache.entries.get(&path)
                        .map(|cached| cached.modified == modified)
                        .unwrap_or(false);

                    if !is_fresh {
                        files_to_process.push((path, modified));
                    } else {
                        unmodified_files.push((path, modified, metadata.len()));
                    }
                }
            }
        }
        
        let process_count = files_to_process.len();
        if process_count == 0 && unmodified_files.is_empty() {
             tracing::info!("Workspace is empty (checked in {:?})", start_total.elapsed());
             return Ok(());
        }

        tracing::info!("Found {} files to process ({} unmodified)", process_count, unmodified_files.len());

        // Snapshot of old hashes for change detection
        let old_hashes: std::collections::HashMap<PathBuf, String> = self.cache.entries
            .iter()
            .map(|(k, v)| (k.clone(), v.hash.clone()))
            .collect();

        // 4. Parallel Parsing Phase (Rayon)
        struct ScanResult {
            path: PathBuf,
            document: Option<cue_common::Document>, // None if content unchanged (only mtime update)
            hash: String,
            modified: SystemTime,
            tokens: usize,
            size_bytes: u64,
        }

        let results: Vec<ScanResult> = files_to_process
            .into_par_iter()
            .filter_map(|(path, modified)| {
                // Read & Hash
                let content = match std::fs::read(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Failed to read {:?}: {}", path, e);
                        return None;
                    }
                };
                
                let size_bytes = content.len() as u64;

                let mut hasher = sha2::Sha256::new();
                hasher.update(&content);
                let hash = format!("{:x}", sha2::Digest::finalize(hasher));

                // Check if content hash matches old hash
                if let Some(old_hash) = old_hashes.get(&path) {
                    if *old_hash == hash {
                        return Some(ScanResult {
                            path,
                            document: None, // Unchanged content
                            hash,
                            modified,
                            tokens: 0, // Will be ignored or retrieved from existing
                            size_bytes,
                        });
                    }
                }

                // Content changed or new file -> Parse
                match parse_file(&path) {
                    Ok(mut doc) => {
                        doc.hash = hash.clone(); // Ensure consistency
                        Some(ScanResult {
                            path,
                            document: Some(doc.clone()),
                            hash,
                            modified,
                            tokens: doc.tokens,
                            size_bytes,
                        })
                    }
                    Err(e) => {
                         tracing::warn!("Failed to parse {:?}: {}", path, e);
                         None
                    }
                }
            })
            .collect();

        // 5. Synchronization Phase (Serial)
        tracing::info!("Syncing {} processed files", results.len());
        
        // Prepare batch for DB
        let mut db_batch = Vec::new();
        // Prepare batch for Cache
        let mut cache_batch = Vec::new();

        for result in results {
            if let Some(doc) = result.document {
                // Full update (New/Modified content)
                self.graph.add_or_update_document(&doc);
                cache_batch.push((result.path.clone(), doc, result.hash.clone(), result.modified));
                db_batch.push((result.path, result.hash, result.modified, result.size_bytes, result.tokens));
            } else {
                 // Content Unchanged (Only mtime updated)
                 // Just need to update mtime in cache and DB
                 // We can use the existing entry from cache to get the doc for graph update?
                 // Actually graph doesn't change if content hash is same.
                 // We need to update cache mtime.
                 if let Some(cached) = self.cache.entries.get_mut(&result.path) {
                     cached.modified = result.modified;
                 }
                 // For DB, we just update modified_at. 
                 // We can reusing upsert, but we need tokens.
                 // We can get tokens from cache.
                 let tokens = self.cache.entries.get(&result.path).map(|d| d.document.tokens).unwrap_or(0);
                 db_batch.push((result.path, result.hash, result.modified, result.size_bytes, tokens));
            }
        }
        
        // Also synchronize unmodified files to DB (in case DB is missing/out-of-sync but Cache is valid)
        for (path, modified, size) in unmodified_files {
            if let Some(entry) = self.cache.entries.get(&path) {
                // We use the cached hash and tokens
                db_batch.push((path, entry.hash.clone(), modified, size, entry.document.tokens));
            }
        }

        // Apply batches
        // Cache Insert
        self.cache.insert_batch(cache_batch);

        // DB Upsert
        if let Some(db) = &mut self.db {
            // Convert to i64 for DB
            let batch_formatted: Vec<_> = db_batch.into_iter().map(|(p, h, m, s, t)| {
                 let mod_timestamp = m.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                 (p, h, mod_timestamp, s, t)
            }).collect();

            if !batch_formatted.is_empty() {
                if let Err(e) = db.upsert_files_batch(&batch_formatted) {
                    tracing::warn!("Failed batch DB update: {}", e);
                }
            }
        }

        // Save cache
        self.cache.save().map_err(|e| std::io::Error::other(e.to_string()))?;

        tracing::info!("Scan completed in {:?}", start_total.elapsed());
        Ok(())
    }

    /// Handle file system event (update/add)
    pub fn update_file(&mut self, path: &Path) -> Result<()> {
        match self.cache.get_or_parse(path) {
            Ok(doc) => {
                // Phase 7.3: Sync to SQLite if available
                if let Some(db) = &self.db {
                    let size_bytes = std::fs::metadata(path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    
                    if let Err(e) = db.upsert_file(path, &doc.hash, size_bytes, doc.tokens) {
                        tracing::warn!("Failed to sync file to DB {:?}: {}", path, e);
                    }
                }

                self.graph.add_or_update_document(&doc);
                self.known_files.insert(path.to_path_buf());
                Ok(())
            }
            Err(e) => {
                // If parsing fails (e.g. file is temporarily locked or invalid),
                // we should probably just warn and not crash
                // But we return error here to let caller decide
                Err(std::io::Error::other(format!("Failed to parse {:?}: {}", path, e)).into())
            }
        }
    }

    /// Handle file deletion
    pub fn remove_file(&mut self, path: &Path) {
        // Phase 7.3: Sync deletion to SQLite if available
        if let Some(db) = &self.db {
            if let Err(e) = db.delete_file(path) {
                tracing::warn!("Failed to delete file from DB {:?}: {}", path, e);
            }
        }

        self.cache.invalidate(path);
        self.graph.remove_document(&path.to_path_buf());
        self.known_files.remove(path);
    }

    /// Render the scene
    pub fn render(&self) -> Result<String> {
        // Topological sort
        let sorted_paths = self.graph.sort_topological()?;

        let mut scene = String::from("# Scene Context\n\n");
        scene.push_str(&format!("Generated: {}\n", chrono::Utc::now().to_rfc3339()));

        // Count documents
        let total_docs = sorted_paths
            .iter()
            .filter(|p| self.cache.get(p).is_some())
            .count();
        scene.push_str(&format!("Documents: {}\n\n", total_docs));

        let mut total_tokens = 0;

        for path in sorted_paths {
            if let Some(doc) = self.cache.get(&path) {
                if total_tokens + doc.tokens > self.config.budgets.feature {
                    tracing::warn!("Token limit reached, truncating scene");
                    break;
                }

                scene.push_str(&format!("## {}\n\n", path.display()));
                scene.push_str(&format!(
                    "Tokens: {} | Hash: {}\n\n",
                    doc.tokens,
                    &doc.hash[..8]
                ));

                // Add anchors
                if !doc.anchors.is_empty() {
                    scene.push_str("### Anchors\n\n");
                    for anchor in &doc.anchors {
                        scene.push_str(&format!(
                            "- {} (L{}): {}\n",
                            "#".repeat(anchor.level as usize),
                            anchor.start_line,
                            anchor.header
                        ));
                    }
                    scene.push('\n');
                }

                // Read content
                if let Ok(content) = std::fs::read_to_string(path) {
                    scene.push_str(&content);
                    scene.push_str("\n\n");
                }

                total_tokens += doc.tokens;
            }
        }

        scene.push_str(&format!("\n---\nTotal Tokens: {}\n", total_tokens));

        Ok(scene)
    }

    /// Get reference to internal cache
    pub fn cache(&self) -> &DocumentCache {
        &self.cache
    }

    /// Get reference to internal graph
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_scan_all_flow() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards = temp.child(".cuedeck/cards");
        cards.create_dir_all().unwrap();
        
        let file1 = cards.child("test1.md");
        file1.write_str("# Test 1\nContent").unwrap();
        
        // Initialize engine (should trigger scan_all)
        let mut engine = CueEngine::new(temp.path()).unwrap();
        
        // internal scan_all called in new()
        assert!(engine.cache.get(file1.path()).is_some());
        assert_eq!(engine.known_files.len(), 1);

        // Add a new file
        let file2 = cards.child("test2.md");
        file2.write_str("# Test 2\nContent").unwrap();
        
        // Update existing file (change content)
        file1.write_str("# Test 1 Modified\nContent").unwrap();
        
        // Wait a bit to ensure mtime diff if filesystem is fast (though we check hash too)
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Scan again
        engine.scan_all().unwrap();

        assert_eq!(engine.known_files.len(), 2);
        assert!(engine.cache.get(file2.path()).is_some());
        
        // Verify content update
        let doc1 = engine.cache.get(file1.path()).unwrap();
        // We can't easily check content directly as doc doesn't store it, 
        // but hash should be different if we tracked it, or just trust mtime/hash logic worked.
        // We can check anchors/headers if we parsed it correctly.
        assert_eq!(doc1.anchors[0].header, "Test 1 Modified");
    }

    #[test]
    fn test_scan_all_deletions() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards = temp.child(".cuedeck/cards");
        cards.create_dir_all().unwrap();
        
        let file1 = cards.child("test1.md");
        file1.write_str("# Test 1").unwrap();
        
        let mut engine = CueEngine::new(temp.path()).unwrap();
        assert_eq!(engine.known_files.len(), 1);

        // Delete file
        // Note: assert_fs doesn't have explicit delete, use std::fs
        std::fs::remove_file(file1.path()).unwrap();

        engine.scan_all().unwrap();
        
        // Check if file is removed from known_files and cache
        assert_eq!(engine.known_files.len(), 0);
        assert!(engine.cache.get(file1.path()).is_none());
    }

    #[test]
    fn test_parallel_scan_correctness() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards = temp.child(".cuedeck/cards");
        cards.create_dir_all().unwrap();

        // Create multiple files
        let file_count = 20;
        for i in 0..file_count {
            let file = cards.child(format!("test_{}.md", i));
            file.write_str(&format!("---\ntitle: Test {}\ntype: feature\n---\n# Heading {}\n\nContent for file {}", i, i, i)).unwrap();
        }

        let engine = CueEngine::new(temp.path()).unwrap();

        // Verify all files are parsed and cached
        assert_eq!(engine.known_files.len(), file_count);
        for i in 0..file_count {
            let path = cards.child(format!("test_{}.md", i));
            let doc = engine.cache.get(path.path());
            assert!(doc.is_some(), "Document {} should be in cache", i);
            let doc = doc.unwrap();
            assert_eq!(doc.anchors.len(), 1);
            assert_eq!(doc.anchors[0].header, format!("Heading {}", i));
        }
    }

    #[test]
    fn test_incremental_update() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards = temp.child(".cuedeck/cards");
        cards.create_dir_all().unwrap();

        // Create initial files
        let file1 = cards.child("file1.md");
        let file2 = cards.child("file2.md");
        let file3 = cards.child("file3.md");
        
        file1.write_str("# File 1").unwrap();
        file2.write_str("# File 2").unwrap();
        file3.write_str("# File 3").unwrap();

        let mut engine = CueEngine::new(temp.path()).unwrap();
        assert_eq!(engine.known_files.len(), 3);

        // Get initial hashes
        let hash1_before = engine.cache.get(file1.path()).unwrap().hash.clone();
        let hash2_before = engine.cache.get(file2.path()).unwrap().hash.clone();
        let hash3_before = engine.cache.get(file3.path()).unwrap().hash.clone();

        // Modify only file2
        std::thread::sleep(std::time::Duration::from_millis(10));
        file2.write_str("# File 2 Modified\n\nNew content").unwrap();

        // Rescan
        engine.scan_all().unwrap();

        // Verify: file1 and file3 should have same hash (not reparsed)
        // file2 should have different hash
        let hash1_after = engine.cache.get(file1.path()).unwrap().hash.clone();
        let hash2_after = engine.cache.get(file2.path()).unwrap().hash.clone();
        let hash3_after = engine.cache.get(file3.path()).unwrap().hash.clone();

        assert_eq!(hash1_before, hash1_after, "File 1 should not be reparsed");
        assert_ne!(hash2_before, hash2_after, "File 2 should be reparsed");
        assert_eq!(hash3_before, hash3_after, "File 3 should not be reparsed");

        // Verify updated content
        let doc2 = engine.cache.get(file2.path()).unwrap();
        assert_eq!(doc2.anchors[0].header, "File 2 Modified");
    }
}


