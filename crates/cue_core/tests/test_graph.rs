//! Graph resolution unit tests

use cue_core::resolve_graph;
use cue_common::Document;
use std::path::PathBuf;

/// Helper function to create a test document
fn create_test_doc(path: &str) -> Document {
    Document {
        path: PathBuf::from(path),
        frontmatter: None,
        hash: "test_hash".to_string(),
        tokens: 100,
        anchors: vec![],
        links: vec![],
    }
}

#[test]
fn test_empty_graph() {
    let docs = vec![];
    let result = resolve_graph(&docs).unwrap();
    assert_eq!(result.len(), 0, "Empty graph should return empty result");
}

#[test]
fn test_single_document() {
    let doc = create_test_doc("test.md");
    let result = resolve_graph(&[doc]).unwrap();
    assert_eq!(result.len(), 1, "Single document should return one node");
}

#[test]
fn test_multiple_independent_documents() {
    let docs = vec![
        create_test_doc("doc1.md"),
        create_test_doc("doc2.md"),
        create_test_doc("doc3.md"),
    ];
    let result = resolve_graph(&docs).unwrap();
    assert_eq!(result.len(), 3, "Should return all independent documents");
}

// TODO: Enable this test when dependency parsing is implemented
#[test]
#[ignore]
fn test_cycle_detection() {
    // This test will be enabled when link parsing is implemented
    // Create A->B->A cycle and verify CycleDetected error
    // Expected: CueError::CycleDetected
}

// TODO: Enable this test when dependency parsing is implemented
#[test]
#[ignore]
fn test_topological_ordering() {
    // This test will verify that dependencies are ordered correctly
    // If A depends on B, then B should appear before A in result
}
