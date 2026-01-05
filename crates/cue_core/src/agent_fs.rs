//! Safe filesystem operations for AI agents
//!
//! This module provides secure file editing capabilities with:
//! - Canonical path validation (prevent path traversal)
//! - File type blocklist (prevent editing sensitive files)
//! - Automatic backups (max 10 versions per file)
//! - Workspace boundary enforcement

use cue_common::{CueError, Result};
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Blocked file extensions (security)
const BLOCKED_EXTENSIONS: &[&str] = &[
    "exe", "dll", "so", "dylib",   // Binaries
    "key", "pem", "p12", "crt",    // Crypto keys/certs
    "env", "secrets",              // Credentials
    "db", "sqlite", "sqlite3",     // Databases (use DB API instead)
];

/// Blocked directory names
const BLOCKED_DIRS: &[&str] = &[
    ".git",
    ".cuedeck",
];

/// Maximum file size for editing (10MB)
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum number of backups to keep per file
const MAX_BACKUPS_PER_FILE: usize = 10;

/// Validate path is within workspace and safe to edit
///
/// # Security Checks
/// - Canonical path validation (prevents ../ attacks)
/// - Workspace boundary enforcement
/// - Blocked directory detection (.git, .cuedeck)
/// - File type blocklist
///
/// # Arguments
/// * `file_path` - Path to validate (may be relative or contain ..)
/// * `workspace` - Workspace root directory
///
/// # Returns
/// Canonical path if valid
pub fn validate_path(file_path: &Path, workspace: &Path) -> Result<PathBuf> {
    // Resolve symlinks and .. segments
    let canonical = file_path
        .canonicalize()
        .or_else(|_| -> Result<PathBuf> {
            // File doesn't exist yet, validate parent directory
            let parent = file_path
                .parent()
                .ok_or_else(|| CueError::ValidationError("Invalid path".into()))?;
            
            let canonical_parent = parent.canonicalize()
                .map_err(|_| CueError::ValidationError("Parent directory does not exist".into()))?;
            
            Ok(canonical_parent.join(
                file_path.file_name()
                    .ok_or_else(|| CueError::ValidationError("Invalid filename".into()))?
            ))
        })?;

    let canonical_workspace = workspace
        .canonicalize()
        .map_err(|_| CueError::ValidationError("Workspace directory does not exist".into()))?;

    // Check if file is within workspace
    if !canonical.starts_with(&canonical_workspace) {
        return Err(CueError::ValidationError(format!(
            "Path traversal blocked: {:?} is outside workspace {:?}",
            file_path, workspace
        )));
    }

    // Block dangerous paths
    for component in canonical.components() {
        if let Some(s) = component.as_os_str().to_str() {
            if BLOCKED_DIRS.contains(&s) {
                return Err(CueError::ValidationError(format!(
                    "Cannot modify files in {} directory",
                    s
                )));
            }
        }
    }

    // Block dangerous file types
    if let Some(ext) = canonical.extension().and_then(|e| e.to_str()) {
        if BLOCKED_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
            return Err(CueError::ValidationError(format!(
                "Editing .{} files is prohibited for security",
                ext
            )));
        }
    }

    // Check file size if file exists
    if canonical.exists() && canonical.is_file() {
        let metadata = fs::metadata(&canonical)?;
        if metadata.len() > MAX_FILE_SIZE {
            return Err(CueError::ValidationError(format!(
                "File too large: {}MB (max: {}MB)",
                metadata.len() / 1024 / 1024,
                MAX_FILE_SIZE / 1024 / 1024
            )));
        }
    }

    Ok(canonical)
}

/// Create backup of file before modification
///
/// # Naming Convention
/// `<filename>_YYYYMMDD_HHMMSS`
///
/// # Cleanup
/// Automatically keeps only last 10 backups per file
fn create_backup(file_path: &Path, workspace: &Path) -> Result<PathBuf> {
    let backup_dir = workspace.join(".cuedeck/backups");
    fs::create_dir_all(&backup_dir)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = file_path
        .file_name()
        .ok_or_else(|| CueError::ValidationError("Invalid filename".into()))?
        .to_str()
        .ok_or_else(|| CueError::ValidationError("Non-UTF8 filename".into()))?;
    
    let backup_path = backup_dir.join(format!("{}_{}", filename, timestamp));

    // Copy file to backup
    fs::copy(file_path, &backup_path)?;
    tracing::info!("Created backup: {:?}", backup_path);

    // Cleanup old backups (keep only last 10)
    cleanup_old_backups(&backup_dir, filename)?;

    Ok(backup_path)
}

/// Cleanup old backups, keeping only the most recent MAX_BACKUPS_PER_FILE
fn cleanup_old_backups(backup_dir: &Path, base_filename: &str) -> Result<()> {
    let mut backups: Vec<_> = fs::read_dir(backup_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|s| s.starts_with(base_filename))
                .unwrap_or(false)
        })
        .collect();

    // Sort by creation time (newest first)
    backups.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.created())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    backups.reverse();

    // Delete excess backups
    for old_backup in backups.into_iter().skip(MAX_BACKUPS_PER_FILE) {
        let _ = fs::remove_file(old_backup.path());
        tracing::debug!("Deleted old backup: {:?}", old_backup.path());
    }

    Ok(())
}

/// Result of replace_in_file operation
#[derive(Debug, Serialize)]
pub struct ReplaceResult {
    /// Relative path from workspace
    pub path: String,
    /// Number of matches found and replaced
    pub matches_found: usize,
    /// Path to backup file (if changes were made)
    pub backup_path: Option<String>,
}

/// Replace text in file with automatic backup
///
/// # Arguments
/// * `workspace` - Workspace root directory
/// * `relative_path` - Path relative to workspace
/// * `find` - Text or regex to find
/// * `replace` - Replacement text
/// * `regex` - If true, treat `find` as regex pattern
///
/// # Returns
/// ReplaceResult with match count and backup path
pub fn replace_in_file(
    workspace: &Path,
    relative_path: &str,
    find: &str,
    replace: &str,
    regex: bool,
) -> Result<ReplaceResult> {
    let file_path = workspace.join(relative_path);
    
    // Security validation
    let canonical_path = validate_path(&file_path, workspace)?;

    // Read current content
    let content = fs::read_to_string(&canonical_path)?;

    // Perform replacement
    let (new_content, count) = if regex {
        let re = regex::Regex::new(find)
            .map_err(|e| CueError::ValidationError(format!("Invalid regex: {}", e)))?;
        let count = re.find_iter(&content).count();
        (re.replace_all(&content, replace).to_string(), count)
    } else {
        let count = content.matches(find).count();
        (content.replace(find, replace), count)
    };

    if count == 0 {
        return Ok(ReplaceResult {
            path: relative_path.to_string(),
            matches_found: 0,
            backup_path: None,
        });
    }

    // Create backup before modifying
    let backup_path = create_backup(&canonical_path, workspace)?;

    // Write new content
    fs::write(&canonical_path, new_content)?;

    Ok(ReplaceResult {
        path: relative_path.to_string(),
        matches_found: count,
        backup_path: Some(backup_path.display().to_string()),
    })
}

/// Read specific line range from file (token-efficient)
///
/// # Arguments
/// * `workspace` - Workspace root directory
/// * `relative_path` - Path relative to workspace
/// * `start_line` - Starting line (1-indexed, inclusive)
/// * `end_line` - Ending line (1-indexed, inclusive)
///
/// # Returns
/// Content of specified line range
pub fn read_file_lines(
    workspace: &Path,
    relative_path: &str,
    start_line: usize,
    end_line: usize,
) -> Result<String> {
    let file_path = workspace.join(relative_path);
    
    // Security validation
    let canonical_path = validate_path(&file_path, workspace)?;

    // Read only requested lines (memory-efficient)
    let file = fs::File::open(&canonical_path)?;
    let reader = io::BufReader::new(file);

    use io::BufRead;
    let lines: Vec<String> = reader
        .lines()
        .skip(start_line.saturating_sub(1))
        .take(end_line - start_line + 1)
        .collect::<io::Result<_>>()?;

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_path_traversal_blocked() {
        let temp = assert_fs::TempDir::new().unwrap();
        let workspace = temp.path();

        // Create a file outside workspace to test absolute path attack
        let outside_file = temp.child("../outside.txt");
        outside_file.write_str("sensitive").unwrap();

        // Attack attempts with existing files
        let result1 = validate_path(&workspace.join("../outside.txt"), workspace);
        assert!(
            result1.is_err(),
            "Path traversal with existing file wasn't blocked"
        );

        // Attack with non-existing file (tests or_else branch)
        let result2 = validate_path(&workspace.join("../../../etc/passwd"), workspace);
        assert!(
            result2.is_err(),
            "Path traversal to non-existing file wasn't blocked"
        );

        // Attack with absolute path
        #[cfg(unix)]
        {
            let result3 = validate_path(&PathBuf::from("/etc/hosts"), workspace);
            assert!(
                result3.is_err(),
                "Absolute path attack wasn't blocked"
            );
        }
        
        #[cfg(windows)]
        {
            let result3 = validate_path(&PathBuf::from("C:\\Windows\\System32\\drivers\\etc\\hosts"), workspace);
            assert!(
                result3.is_err(),
                "Absolute path attack wasn't blocked"
            );
        }

        // Attack with subdirectory escape
        let subdir = temp.child("subdir");
        subdir.create_dir_all().unwrap();
        let result4 = validate_path(&subdir.path().join("../../outside.txt"), workspace);
        assert!(
            result4.is_err(),
            "Subdirectory escape attack wasn't blocked"
        );
    }

    #[test]
    fn test_blocked_directories() {
        let temp = assert_fs::TempDir::new().unwrap();
        
        // Create .git directory
        let git_dir = temp.child(".git");
        git_dir.create_dir_all().unwrap();
        let git_file = git_dir.child("config");
        git_file.touch().unwrap();

        let result = validate_path(git_file.path(), temp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".git"));
    }

    #[test]
    fn test_dangerous_file_types_blocked() {
        let temp = assert_fs::TempDir::new().unwrap();

        let blocked = vec![
            "credentials.env",
            "private.key",
            "app.exe",
            "data.db",
        ];

        for file in blocked {
            let test_file = temp.child(file);
            test_file.touch().unwrap();
            
            let result = validate_path(test_file.path(), temp.path());
            assert!(result.is_err(), "Dangerous file not blocked: {}", file);
        }
    }

    #[test]
    fn test_replace_in_file_basic() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.txt");
        file.write_str("Hello World\nHello Rust").unwrap();

        let result = replace_in_file(
            temp.path(),
            "test.txt",
            "Hello",
            "Hi",
            false,
        )
        .unwrap();

        assert_eq!(result.matches_found, 2);
        assert!(result.backup_path.is_some());

        let content = fs::read_to_string(file.path()).unwrap();
        assert_eq!(content, "Hi World\nHi Rust");
    }

    #[test]
    fn test_replace_with_regex() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.rs");
        file.write_str("pub fn test() {}\npub fn demo() {}").unwrap();

        let result = replace_in_file(
            temp.path(),
            "test.rs",
            r"fn\s+(\w+)",
            "function $1",
            true,
        )
        .unwrap();

        assert_eq!(result.matches_found, 2);
        
        let content = fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("function test"));
        assert!(content.contains("function demo"));
    }

    #[test]
    fn test_backup_creation_and_cleanup() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.txt");
        file.write_str("version 1").unwrap();

        // Create 15 backups (should keep only last 10)
        for i in 1..=15 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            file.write_str(&format!("version {}", i)).unwrap();
            
            replace_in_file(
                temp.path(),
                "test.txt",
                &format!("version {}", i),
                &format!("version {}", i + 1),
                false,
            )
            .unwrap();
        }

        // Check backup count
        let backup_dir = temp.child(".cuedeck/backups");
        let backup_count = fs::read_dir(backup_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .unwrap()
                    .starts_with("test.txt")
            })
            .count();

        assert!(
            backup_count <= MAX_BACKUPS_PER_FILE,
            "Too many backups: {}",
            backup_count
        );
    }

    #[test]
    fn test_read_file_lines_partial() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.txt");
        file.write_str("line 1\nline 2\nline 3\nline 4\nline 5").unwrap();

        let content = read_file_lines(temp.path(), "test.txt", 2, 4).unwrap();
        assert_eq!(content, "line 2\nline 3\nline 4");
    }

    #[test]
    fn test_replace_no_matches() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.txt");
        file.write_str("Hello World").unwrap();

        let result = replace_in_file(
            temp.path(),
            "test.txt",
            "Goodbye",
            "Hi",
            false,
        )
        .unwrap();

        assert_eq!(result.matches_found, 0);
        assert!(result.backup_path.is_none()); // No backup if no changes
    }

    #[test]
    fn test_file_size_limit() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("large.txt");
        
        // Create file larger than MAX_FILE_SIZE
        let large_content = "x".repeat((MAX_FILE_SIZE + 1) as usize);
        file.write_str(&large_content).unwrap();

        let result = validate_path(file.path(), temp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }
}
