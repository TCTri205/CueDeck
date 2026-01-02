use cue_core::context::{search_workspace_with_mode, SearchFilters, SearchMode};
use assert_fs::prelude::*;

#[test]
fn test_search_filters_integration() {
    let temp = assert_fs::TempDir::new().unwrap();
    let root = temp.path();

    // Setup files
    temp.child("doc1.md")
        .write_str(r#"---
title: Doc 1
priority: high
assignee: "@alice"
tags: ["auth", "security"]
---
# Doc 1
Authentication system"#)
        .unwrap();

    temp.child("doc2.md")
        .write_str(r#"---
title: Doc 2
priority: low
assignee: "@bob"
tags: ["ui"]
---
# Doc 2
User interface"#)
        .unwrap();

    temp.child("doc3.md")
        .write_str(r#"---
title: Doc 3
priority: high
assignee: "@alice"
tags: ["api", "backend"]
---
# Doc 3
Backend API"#)
        .unwrap();

    // Test 1: Tag Filter (ANY match)
    // Filter tags=["auth"] -> should match doc1
    let filters = SearchFilters {
        tags: Some(vec!["auth".to_string()]),
        ..Default::default()
    };
    let results = search_workspace_with_mode(root, "doc", SearchMode::Keyword, Some(filters)).unwrap();
    assert_eq!(results.len(), 1, "Should find 1 doc with tag 'auth'");
    assert_eq!(results[0].frontmatter.as_ref().unwrap().title, "Doc 1");

    // Test 2: Priority Filter
    // Filter priority="low" -> should match doc2
    let filters = SearchFilters {
        priority: Some("low".to_string()),
        ..Default::default()
    };
    let results = search_workspace_with_mode(root, "doc", SearchMode::Keyword, Some(filters)).unwrap();
    assert_eq!(results.len(), 1, "Should find 1 doc with priority 'low'");
    assert_eq!(results[0].frontmatter.as_ref().unwrap().title, "Doc 2");

    // Test 3: Assignee Filter
    // Filter assignee="@alice" -> should match doc1 and doc3
    let filters = SearchFilters {
        assignee: Some("@alice".to_string()),
        ..Default::default()
    };
    let results = search_workspace_with_mode(root, "doc", SearchMode::Keyword, Some(filters)).unwrap();
    assert_eq!(results.len(), 2, "Should find 2 docs with assignee '@alice'");
    
    // Test 4: Combined Filter
    // Filter priority="high" AND assignee="@alice" -> match doc1 and doc3
    let filters = SearchFilters {
        priority: Some("high".to_string()),
        assignee: Some("@alice".to_string()),
        ..Default::default()
    };
    let results = search_workspace_with_mode(root, "doc", SearchMode::Keyword, Some(filters)).unwrap();
    assert_eq!(results.len(), 2);

    // Filter priority="high" AND tags=["ui"] -> match nothing
    let filters = SearchFilters {
        priority: Some("high".to_string()),
        tags: Some(vec!["ui".to_string()]),
        ..Default::default()
    };
    let results = search_workspace_with_mode(root, "doc", SearchMode::Keyword, Some(filters)).unwrap();
    assert_eq!(results.len(), 0);
    
    // Test 5: Case Insensitivity
    let filters = SearchFilters {
        tags: Some(vec!["AUTH".to_string()]), // Uppercase
        ..Default::default()
    };
    let results = search_workspace_with_mode(root, "doc", SearchMode::Keyword, Some(filters)).unwrap();
    assert_eq!(results.len(), 1, "Should handle case insensitivity");
}
