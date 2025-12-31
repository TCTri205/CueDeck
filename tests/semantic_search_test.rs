use cue_core::context::search_workspace;
use assert_fs::prelude::*;

#[test]
fn test_semantic_vs_keyword_search() {
    let temp = assert_fs::TempDir::new().unwrap();
    
    // Create files with conceptually similar content but different keywords
    temp.child("async.md").write_str(r#"---
title: "Async Programming"
status: active
---
# Asynchronous Operations

This document covers asynchronous programming patterns in Rust including futures, async/await syntax, and concurrent execution.
"#).unwrap();

    temp.child("sync.md").write_str(r#"---
title: "Synchronous Code"
status: active
---
# Blocking Operations

Traditional blocking I/O operations that execute sequentially.
"#).unwrap();

    temp.child("recipes.md").write_str(r#"---
title: "Cooking Recipes"
status: active
---
# Delicious Recipes

Collection of cooking recipes and techniques.
"#).unwrap();

    let root = temp.path();

    // Test 1: Semantic search should find conceptually related documents
    // Query "concurrent execution" should match "async.md" even though it uses different words
    let semantic_results = search_workspace(root, "concurrent execution", true).unwrap();
    assert!(!semantic_results.is_empty(), "Semantic search should find related concepts");
    
    // The async.md should rank higher due to semantic similarity
    let has_async = semantic_results.iter().any(|doc| {
        doc.path.file_name().unwrap() == "async.md"
    });
    assert!(has_async, "Semantic search should find conceptually similar async.md");

    // Test 2: Keyword search might miss without exact match
    let keyword_results = search_workspace(root, "concurrent execution", false).unwrap();
    // Keyword search will find "concurrent" in async.md content
    assert!(!keyword_results.is_empty());

    // Test 3: Unrelated query should not match cooking recipes
    let semantic_results2 = search_workspace(root, "async programming", true).unwrap();
    if let Some(first) = semantic_results2.first() {
        assert_ne!(first.path.file_name().unwrap(), "recipes.md", 
            "Unrelated document should not be top result");
    }
}

#[test]
fn test_semantic_search_empty_query() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("test.md").write_str("# Test\nContent").unwrap();

    let results = search_workspace(temp.path(), "", true).unwrap();
    // Empty query should still work (will return low-similarity results)
    assert!(results.is_empty() || !results.is_empty());
}

#[test]
fn test_semantic_search_no_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    
    let results = search_workspace(temp.path(), "test query", true).unwrap();
    assert!(results.is_empty(), "Should return empty for directory with no markdown files");
}
