use assert_fs::prelude::*;
use cue_core::context::{search_workspace_paginated, SearchMode};

#[test]
fn test_pagination_with_mock_docs() {
    // Create test workspace with 25 markdown files
    let temp = assert_fs::TempDir::new().unwrap();
    
    for i in 0..25 {
        temp.child(format!("doc_{:02}.md", i))
            .write_str(&format!("---\nid: doc{}\n---\n\n# Document {}\nTest content for searching", i, i))
            .unwrap();
    }

    let root = temp.path();

    // Page 1: limit=10, no cursor
    let page1 = search_workspace_paginated(
        root,
        "Document",
        SearchMode::Keyword,
        None,
        10,
        None,
    ).unwrap();

    assert_eq!(page1.docs.len(), 10, "First page should have 10 docs");
    assert_eq!(page1.total_count, 25, "Total count should be 25");
    assert!(page1.next_cursor.is_some(), "Should have next cursor");

    // Page 2: use cursor from page 1
    let page2 = search_workspace_paginated(
        root,
        "Document",
        SearchMode::Keyword,
        None,
        10,
        page1.next_cursor.as_deref(),
    ).unwrap();

    assert_eq!(page2.docs.len(), 10, "Second page should have 10 docs");
    assert_eq!(page2.total_count, 25, "Total count should remain 25");
    assert!(page2.next_cursor.is_some(), "Should have next cursor for page 3");

    // Verify different documents
    let page1_paths: Vec<_> = page1.docs.iter().map(|d| &d.path).collect();
    let page2_paths: Vec<_> = page2.docs.iter().map(|d| &d.path).collect();
    
    for p2_path in &page2_paths {
        assert!(!page1_paths.contains(p2_path), "Pages should not overlap");
    }

    // Page 3: last page
    let page3 = search_workspace_paginated(
        root,
        "Document",
        SearchMode::Keyword,
        None,
        10,
        page2.next_cursor.as_deref(),
    ).unwrap();

    assert_eq!(page3.docs.len(), 5, "Last page should have 5 remaining docs");
    assert_eq!(page3.total_count, 25, "Total count should remain 25");
    assert!(page3.next_cursor.is_none(), "Last page should have no next cursor");
}

#[test]
fn test_pagination_empty_results() {
    let temp = assert_fs::TempDir::new().unwrap();
    
    temp.child("unrelated.md")
        .write_str("# Unrelated\nNothing to find here")
        .unwrap();

    let root = temp.path();

    let result = search_workspace_paginated(
        root,
        "nonexistent query",
        SearchMode::Keyword,
        None,
        10,
        None,
    ).unwrap();

    assert_eq!(result.docs.len(), 0, "Should return empty results");
    assert_eq!(result.total_count, 0, "Total count should be 0");
    assert!(result.next_cursor.is_none(), "No cursor for empty results");
}

#[test]
fn test_pagination_single_page() {
    let temp = assert_fs::TempDir::new().unwrap();
    
    for i in 0..5 {
        temp.child(format!("doc{}.md", i))
            .write_str(&format!("# Doc {}\nSample content", i))
            .unwrap();
    }

    let root = temp.path();

    let result = search_workspace_paginated(
        root,
        "Doc",
        SearchMode::Keyword,
        None,
        10,
        None,
    ).unwrap();

    assert_eq!(result.docs.len(), 5, "Should return all 5 docs");
    assert_eq!(result.total_count, 5, "Total count should be 5");
    assert!(result.next_cursor.is_none(), "No next page needed");
}

#[test]
fn test_pagination_exact_page_boundary() {
    let temp = assert_fs::TempDir::new().unwrap();
    
    // Exactly 20 docs
    for i in 0..20 {
        temp.child(format!("task{}.md", i))
            .write_str(&format!("# Task {}\nContent here", i))
            .unwrap();
    }

    let root = temp.path();

    // Page 1: 10 docs
    let page1 = search_workspace_paginated(
        root,
        "Task",
        SearchMode::Keyword,
        None,
        10,
        None,
    ).unwrap();

    assert_eq!(page1.docs.len(), 10);
    assert_eq!(page1.total_count, 20);
    assert!(page1.next_cursor.is_some());

    // Page 2: exactly 10 docs, no more
    let page2 = search_workspace_paginated(
        root,
        "Task",
        SearchMode::Keyword,
        None,
        10,
        page1.next_cursor.as_deref(),
    ).unwrap();

    assert_eq!(page2.docs.len(), 10);
    assert_eq!(page2.total_count, 20);
    assert!(page2.next_cursor.is_none(), "Should be last page");
}

#[test]
fn test_pagination_different_limits() {
    let temp = assert_fs::TempDir::new().unwrap();
    
    for i in 0..15 {
        temp.child(format!("file{}.md", i))
            .write_str(&format!("# File {}", i))
            .unwrap();
    }

    let root = temp.path();

    // Test with limit=5
    let page1 = search_workspace_paginated(
        root,
        "File",
        SearchMode::Keyword,
        None,
        5,
        None,
    ).unwrap();

    assert_eq!(page1.docs.len(), 5);
    assert_eq!(page1.total_count, 15);
    assert!(page1.next_cursor.is_some());

    // Test with limit=3 from offset 5
    let page2 = search_workspace_paginated(
        root,
        "File",
        SearchMode::Keyword,
        None,
        3,
        page1.next_cursor.as_deref(),
    ).unwrap();

    assert_eq!(page2.docs.len(), 3);
    assert_eq!(page2.total_count, 15);
}
