use cue_core::{parse_file, tasks, resolve_graph};
use cue_common::CardMetadata;
use std::fs;
use std::path::Path;
use assert_fs::prelude::*;
use serde_json::json;

#[test]
fn test_core_parsing_and_graph() {
    let temp = assert_fs::TempDir::new().unwrap();
    
    // Create File A
    let file_a = temp.child("file_a.md");
    file_a.write_str(r#"---
title: "File A"
status: active
---
# Header in A

Link to [[file-b]]
"#).unwrap();

    // Create File B
    let file_b = temp.child("file_b.md");
    file_b.write_str(r#"---
title: "File B"
status: todo
---
# Header in B
"#).unwrap();

    // Parse
    let doc_a = parse_file(file_a.path()).expect("Failed to parse A");
    let doc_b = parse_file(file_b.path()).expect("Failed to parse B");

    // Verify Metadata
    assert_eq!(doc_a.frontmatter.as_ref().unwrap().title, "File A");
    assert_eq!(doc_a.frontmatter.as_ref().unwrap().status, "active");
    
    // Verify Links
    assert!(doc_a.links.contains(&"file-b".to_string()));

    // Verify Graph
    let docs = vec![doc_a, doc_b];
    let sorted = resolve_graph(&docs).expect("Graph resolution failed");
    
    // B should come before A because A depends on B
    assert_eq!(sorted[0].file_name().unwrap(), "file_b.md");
    assert_eq!(sorted[1].file_name().unwrap(), "file_a.md");
}

#[test]
fn test_task_management() {
    let temp = assert_fs::TempDir::new().unwrap();
    let root = temp.path();
    
    // Create Task
    let task_path = tasks::create_task(root, "New Task").expect("Create failed");
    assert!(task_path.exists());
    
    // List Tasks
    let list = tasks::list_tasks(root, None, None).expect("List failed");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].frontmatter.as_ref().unwrap().title, "New Task");
    
    // Update Task
    let id = task_path.file_stem().unwrap().to_str().unwrap();
    let updates = json!({
        "status": "done",
        "priority": "high"
    });
    
    let updated_doc = tasks::update_task(root, id, updates.as_object().unwrap().clone()).expect("Update failed");
    assert_eq!(updated_doc.frontmatter.as_ref().unwrap().status, "done");
    assert_eq!(updated_doc.frontmatter.as_ref().unwrap().priority, "high");
    
    // Verify persistence
    let content = fs::read_to_string(&task_path).unwrap();
    assert!(content.contains("status: done"));
}

#[test]
fn test_context_search_integration() {
    use cue_core::context::search_workspace;
    let temp = assert_fs::TempDir::new().unwrap();
    let root = temp.path();

    // Setup files
    temp.child("guide.md").write_str("# User Guide\nHow to use the system").unwrap();
    temp.child("api.md").write_str("# API Docs\nReference for developers").unwrap();
    temp.child("ignored.txt").write_str("Not a markdown file").unwrap();

    // 1. Search by filename
    let results = search_workspace(root, "guide").expect("Search failed");
    assert!(!results.is_empty());
    assert_eq!(results[0].path.file_name().unwrap(), "guide.md");

    // 2. Search by content
    let results = search_workspace(root, "developers").expect("Search failed");
    assert!(!results.is_empty());
    assert_eq!(results[0].path.file_name().unwrap(), "api.md");

    // 3. Search miss
    let results = search_workspace(root, "banana").expect("Search failed");
    assert!(results.is_empty());
}
