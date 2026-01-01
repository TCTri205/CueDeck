//! Graph resolution unit tests

use cue_common::Document;
use cue_core::{graph::DependencyGraph, resolve_graph};
use std::path::PathBuf;

/// Helper function to create a test document
fn create_test_doc(path: &str, links: Vec<String>) -> Document {
    Document {
        path: PathBuf::from(path),
        frontmatter: None,
        hash: "test_hash".to_string(),
        tokens: 100,
        anchors: vec![],
        links,
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
    let doc = create_test_doc("test.md", vec![]);
    let result = resolve_graph(&[doc]).unwrap();
    assert_eq!(result.len(), 1, "Single document should return one node");
}

#[test]
fn test_multiple_independent_documents() {
    let docs = vec![
        create_test_doc("doc1.md", vec![]),
        create_test_doc("doc2.md", vec![]),
        create_test_doc("doc3.md", vec![]),
    ];
    let result = resolve_graph(&docs).unwrap();
    assert_eq!(result.len(), 3, "Should return all independent documents");
}

#[test]
fn test_cycle_detection() {
    // Create A->B->C->A cycle
    let docs = vec![
        create_test_doc("a.md", vec!["b.md".to_string()]),
        create_test_doc("b.md", vec!["c.md".to_string()]),
        create_test_doc("c.md", vec!["a.md".to_string()]),
    ];

    let graph = DependencyGraph::build(&docs).unwrap();

    // Should detect cycle
    let cycle = graph.detect_cycle();
    assert!(cycle.is_some(), "Should detect cycle in A->B->C->A");

    // Cycle path should have at least 3 nodes
    let cycle_path = cycle.unwrap();
    assert!(
        cycle_path.len() >= 3,
        "Cycle should contain at least 3 nodes"
    );
}

#[test]
fn test_topological_ordering() {
    // Create A->B, B->C (linear dependency chain)
    let docs = vec![
        create_test_doc("a.md", vec!["b.md".to_string()]),
        create_test_doc("b.md", vec!["c.md".to_string()]),
        create_test_doc("c.md", vec![]),
    ];

    let graph = DependencyGraph::build(&docs).unwrap();

    // Should successfully sort
    let sorted = graph.sort_topological().unwrap();
    assert_eq!(sorted.len(), 3);

    // C should come before B, B should come before A
    let c_idx = sorted
        .iter()
        .position(|p| p.to_str().unwrap() == "c.md")
        .unwrap();
    let b_idx = sorted
        .iter()
        .position(|p| p.to_str().unwrap() == "b.md")
        .unwrap();
    let a_idx = sorted
        .iter()
        .position(|p| p.to_str().unwrap() == "a.md")
        .unwrap();

    assert!(c_idx < b_idx, "c.md should come before b.md");
    assert!(b_idx < a_idx, "b.md should come before a.md");
}
