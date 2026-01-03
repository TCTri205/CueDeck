//! Workspace initialization utilities for tests
//!
//! Provides functions to create temporary directories and initialize
//! CueDeck workspaces for integration testing.

use assert_fs::TempDir;
use std::fs;


/// Create a temporary directory for testing
///
/// The directory will be automatically cleaned up when the `TempDir` is dropped.
///
/// # Example
///
/// ```rust
/// use cue_test_helpers::workspace::temp_dir;
///
/// let temp = temp_dir();
/// // Use temp.path() for testing
/// // Automatically cleaned up on drop
/// ```
pub fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Initialize a CueDeck workspace in a temp directory
///
/// Creates a `.cuedeck` directory with basic structure.
///
/// # Example
///
/// ```rust
/// use cue_test_helpers::workspace::init_workspace;
///
/// let workspace = init_workspace();
/// assert!(workspace.path().join(".cuedeck").exists());
/// ```
pub fn init_workspace() -> TempDir {
    let temp = temp_dir();
    let cuedeck_dir = temp.path().join(".cuedeck");
    fs::create_dir_all(&cuedeck_dir).expect("Failed to create .cuedeck directory");
    
    // Create cache directory
    let cache_dir = cuedeck_dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");
    
    temp
}

/// Create a workspace with predefined task cards
///
/// # Arguments
///
/// * `cards` - Array of tuples (filename, content) to create in the workspace
///
/// # Example
///
/// ```rust
/// use cue_test_helpers::workspace::workspace_with_cards;
///
/// let cards = vec![
///     ("task1.md", "---\nid: abc123\nstatus: todo\n---\n# Task 1"),
///     ("task2.md", "---\nid: def456\nstatus: active\n---\n# Task 2"),
/// ];
/// let workspace = workspace_with_cards(&cards);
/// ```
pub fn workspace_with_cards(cards: &[(&str, &str)]) -> TempDir {
    let workspace = init_workspace();
    
    for (filename, content) in cards {
        let file_path = workspace.path().join(filename);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
        fs::write(&file_path, content).expect("Failed to write card file");
    }
    
    workspace
}

/// Create a markdown file in a temp directory
///
/// Legacy helper for compatibility with existing tests.
///
/// # Arguments
///
/// * `content` - The markdown content to write
///
/// # Returns
///
/// Tuple of (TempDir, PathBuf) where PathBuf is the path to the created file
pub fn create_temp_md(content: &str) -> (TempDir, std::path::PathBuf) {
    let temp = temp_dir();
    let file_path = temp.path().join("test.md");
    fs::write(&file_path, content).expect("Failed to write temp markdown file");
    (temp, file_path)
}
