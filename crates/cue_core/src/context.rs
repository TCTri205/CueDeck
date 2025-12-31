use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use cue_common::{Document, Result, CueError};
use crate::parse_file;

#[tracing::instrument(skip(root))]
pub fn search_workspace(root: &Path, query: &str) -> Result<Vec<Document>> {
    tracing::info!("Searching workspace: {:?} for '{}'", root, query);

    let query_lower = query.to_lowercase();
    let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();
    
    let mut results: Vec<(Document, i32)> = Vec::new();

    // Use walkdir to traverse
    // Filter ignored directories/files
    let walker = WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_ignored(e));

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                 if ext == "md" {
                     let path = entry.path();
                     // Calculate score
                     // We read the file content here for scoring content matches
                     let score = score_file(path, &query_lower, &query_tokens);
                     
                     if score > 0 {
                         match parse_file(path) {
                             Ok(doc) => results.push((doc, score)),
                             Err(e) => tracing::warn!("Failed to parse {:?}: {}", path, e),
                         }
                     }
                 }
            }
        }
    }

    // Sort by score descending
    results.sort_by(|a, b| b.1.cmp(&a.1));

    // Return top 10
    let top_docs = results.into_iter()
        .take(10)
        .map(|(doc, _)| doc)
        .collect();
        
    Ok(top_docs)
}

fn is_ignored(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    // Ignore common noise directories
    name == "node_modules" || 
    name == ".git" || 
    name == "target" || 
    name == "dist" ||
    name == "vendor"
}

fn score_file(path: &Path, query_lower: &str, tokens: &[&str]) -> i32 {
    let mut score = 0;
    let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();

    // Filename match (High weight)
    if file_name.contains(query_lower) {
        score += 100;
    }
    
    // Partial filename token match
    for token in tokens {
        if file_name.contains(token) {
            score += 10;
        }
    }

    // Content match
    // Read file content - ignore errors (treat as empty/unreadable)
    if let Ok(content) = std::fs::read_to_string(path) {
        let content_lower = content.to_lowercase();
        // Exact query match in content
        if content_lower.contains(query_lower) {
            score += 50;
        }
        
        // Token matches
        for token in tokens {
            if content_lower.contains(token) {
                score += 5;
            }
        }
    }
    
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_search_workspace() {
        let temp = assert_fs::TempDir::new().unwrap();
        
        temp.child("readme.md").write_str("# Read Me\nWelcome to CueDeck").unwrap();
        temp.child("todo.md").write_str("# Todo\n- [ ] Task 1").unwrap();
        
        // Ignored file
        temp.child("node_modules").create_dir_all().unwrap();
        temp.child("node_modules/package.json").write_str("{}").unwrap();

        // Create context match file
        temp.child("notes.md").write_str("# Notes\nSome important legacy code").unwrap();

        let root = temp.path();

        // Search for "todo"
        let results = search_workspace(root, "todo").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path.file_name().unwrap().to_str().unwrap(), "todo.md");

        // Search for "legacy" (content match)
        let results = search_workspace(root, "legacy").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path.file_name().unwrap().to_str().unwrap(), "notes.md");
        
        // Search "readme" (filename match)
        let results = search_workspace(root, "readme").unwrap();
         assert_eq!(results.len(), 1);
         assert_eq!(results[0].path.file_name().unwrap(), "readme.md");
    }
}
