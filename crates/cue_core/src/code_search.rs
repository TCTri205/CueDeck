//! Code search using the grep crate (same engine as ripgrep)
//!
//! This module provides token-efficient code search functionality for AI agents.
//! Results include file path + line number + preview (max 100 chars), not full content.

use cue_common::{CueError, Result};
use grep::matcher::Matcher;
use grep::regex::RegexMatcherBuilder;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};
use ignore::WalkBuilder;
use serde::Serialize;
use std::path::Path;

/// Code search result (token-optimized for MCP)
#[derive(Debug, Serialize, Clone)]
pub struct CodeMatch {
    /// Relative path from workspace
    pub path: String,

    /// Line number (1-indexed)
    pub line_number: u64,

    /// Preview context (max 100 chars to save tokens)
    pub preview: String,

    /// Column offset where match starts
    pub column: usize,
}

/// Search code in workspace using regex pattern
///
/// # Arguments
/// * `workspace` - Workspace root directory
/// * `pattern` - Regex pattern to search for (e.g., "fn\\s+authenticate")
/// * `file_glob` - Optional glob filter (e.g., "*.rs", "src/**/*.ts")
/// * `case_sensitive` - Enable case-sensitive matching
/// * `max_results` - Maximum number of results to return (default: 50, max: 100)
///
/// # Returns
/// Vector of matches limited to `max_results`
///
/// # Examples
/// ```no_run
/// use cue_core::code_search::search_code;
/// use std::path::Path;
///
/// let matches = search_code(
///     Path::new("/workspace"),
///     r"fn\s+\w+",  // Find function definitions
///     Some("*.rs"),  // Only Rust files
///     false,         // Case insensitive
///     50             // Max 50 results
/// ).unwrap();
/// ```
pub fn search_code(
    workspace: &Path,
    pattern: &str,
    file_glob: Option<&str>,
    case_sensitive: bool,
    max_results: usize,
) -> Result<Vec<CodeMatch>> {
    // Build regex matcher (same as ripgrep)
    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(!case_sensitive)
        .line_terminator(Some(b'\n'))
        .build(pattern)
        .map_err(|e| CueError::ValidationError(format!("Invalid regex pattern: {}", e)))?;

    // Build searcher with ripgrep-like settings
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(true)
        .build();

    let mut results = Vec::new();
    let max_results = max_results.min(100); // Cap at 100

    // Walk directory respecting .gitignore (same as rg)
    let mut walker = WalkBuilder::new(workspace);
    walker
        .hidden(false) // Don't automatically skip hidden files
        .git_ignore(true) // Respect .gitignore
        .git_global(true) // Respect global gitignore
        .git_exclude(true); // Respect .git/info/exclude

    if let Some(glob) = file_glob {
        // Add glob override (include only matching files)
        walker.add_custom_ignore_filename(glob);
    }

    for entry in walker.build() {
        let entry = match entry {
            Ok(e) if e.file_type().map_or(false, |ft| ft.is_file()) => e,
            _ => continue,
        };

        let path = entry.path();

        // Search file
        let search_result = searcher.search_path(
            &matcher,
            path,
            UTF8(|line_num, line| {
                // Token-efficient: Only store preview (max 100 chars)
                let preview = line.trim();
                let preview = if preview.len() > 100 {
                    format!("{}...", &preview[..97])
                } else {
                    preview.to_string()
                };

                // Find column offset
                let column = matcher
                    .find(line.as_bytes())
                    .ok()
                    .flatten()
                    .map(|m: grep::matcher::Match| m.start())
                    .unwrap_or(0);

                results.push(CodeMatch {
                    path: path
                        .strip_prefix(workspace)
                        .unwrap_or(path)
                        .display()
                        .to_string()
                        .replace('\\', "/"), // Normalize path separators
                    line_number: line_num,
                    preview,
                    column,
                });

                // Stop if we've hit max results
                if results.len() >= max_results {
                    Ok(false) // Stop searching this file
                } else {
                    Ok(true) // Continue
                }
            }),
        );

        // Handle search errors gracefully
        if let Err(e) = search_result {
            tracing::warn!("Failed to search {:?}: {}", path, e);
        }

        // Break if we've collected enough results
        if results.len() >= max_results {
            break;
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_search_code_basic_regex() {
        let temp = assert_fs::TempDir::new().unwrap();
        let src = temp.child("src");
        src.create_dir_all().unwrap();

        let lib_file = src.child("lib.rs");
        lib_file
            .write_str("pub fn authenticate() {}\npub fn verify() {}")
            .unwrap();

        let results = search_code(temp.path(), r"fn\s+\w+", None, false, 50).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].path, "src/lib.rs");
        assert_eq!(results[0].line_number, 1);
        assert!(results[0].preview.contains("authenticate"));
    }

    #[test]
    fn test_search_respects_gitignore() {
        let temp = assert_fs::TempDir::new().unwrap();

        // Initialize git repo (required for gitignore to work)
        let git_dir = temp.child(".git");
        git_dir.create_dir_all().unwrap();

        // Create .gitignore
        let gitignore = temp.child(".gitignore");
        gitignore.write_str("target/\n").unwrap();

        // Create ignored file
        let target_dir = temp.child("target");
        target_dir.create_dir_all().unwrap();
        let ignored_file = target_dir.child("debug.log");
        ignored_file.write_str("sensitive data").unwrap();

        // Create non-ignored file
        let src = temp.child("src");
        src.create_dir_all().unwrap();
        let visible_file = src.child("main.rs");
        visible_file.write_str("normal code").unwrap();

        let results = search_code(temp.path(), "data", None, false, 50).unwrap();

        // Should not find match in ignored file
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_case_sensitivity() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.txt");
        file.write_str("Hello World\nhello world").unwrap();

        // Case insensitive (default)
        let results_insensitive = search_code(temp.path(), "HELLO", None, false, 50).unwrap();
        assert_eq!(results_insensitive.len(), 2);

        // Case sensitive
        let results_sensitive = search_code(temp.path(), "HELLO", None, true, 50).unwrap();
        assert_eq!(results_sensitive.len(), 0); // No match
    }

    #[test]
    fn test_search_max_results_limit() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("many_matches.txt");

        // Create file with 100 lines containing "match"
        let content = (0..100)
            .map(|i| format!("Line {} with match keyword", i))
            .collect::<Vec<_>>()
            .join("\n");
        file.write_str(&content).unwrap();

        // Request max 10 results
        let results = search_code(temp.path(), "match", None, false, 10).unwrap();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_search_invalid_regex() {
        let temp = assert_fs::TempDir::new().unwrap();

        let result = search_code(temp.path(), "[invalid(regex", None, false, 50);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid regex pattern"));
    }

    #[test]
    fn test_search_preview_truncation() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("long_line.txt");

        // Create line longer than 100 chars
        let long_line = "x".repeat(150);
        file.write_str(&long_line).unwrap();

        let results = search_code(temp.path(), "x+", None, false, 50).unwrap();
        assert_eq!(results.len(), 1);

        // Preview should be truncated to ~100 chars
        assert!(results[0].preview.len() <= 100);
        assert!(results[0].preview.ends_with("..."));
    }

    #[test]
    fn test_search_column_offset() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.rs");
        file.write_str("    pub fn test() {}").unwrap();

        let results = search_code(temp.path(), "fn", None, false, 50).unwrap();
        assert_eq!(results.len(), 1);

        // Column should point to start of "fn" (after "pub ")
        assert_eq!(results[0].column, 8); // "    pub " = 8 chars
    }
}
