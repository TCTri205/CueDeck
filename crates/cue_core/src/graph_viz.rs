//! Graph visualization module
//!
//! Provides rendering of dependency graphs into various formats:
//! - Mermaid (for GitHub/docs)
//! - DOT (for Graphviz)
//! - ASCII (terminal output)
//! - JSON (machine-readable)

use crate::graph::DependencyGraph;
use petgraph::visit::EdgeRef;
use serde_json::json;

/// Supported visualization formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphFormat {
    /// Mermaid flowchart format
    Mermaid,
    /// Graphviz DOT format
    Dot,
    /// ASCII tree for terminal
    Ascii,
    /// JSON structure
    Json,
}

impl std::str::FromStr for GraphFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mermaid" => Ok(Self::Mermaid),
            "dot" => Ok(Self::Dot),
            "ascii" => Ok(Self::Ascii),
            "json" => Ok(Self::Json),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

/// Render a dependency graph to string
pub fn render(graph: &DependencyGraph, format: GraphFormat) -> String {
    match format {
        GraphFormat::Mermaid => render_mermaid(graph),
        GraphFormat::Dot => render_dot(graph),
        GraphFormat::Ascii => render_ascii(graph),
        GraphFormat::Json => render_json(graph),
    }
}

fn render_mermaid(graph: &DependencyGraph) -> String {
    let mut output = String::from("```mermaid\nflowchart LR\n");

    // Get all nodes
    let nodes: Vec<_> = graph
        .graph
        .node_indices()
        .map(|idx| &graph.graph[idx])
        .collect();

    // Generate node definitions
    for (i, node) in nodes.iter().enumerate() {
        let name = node
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        output.push_str(&format!("    N{}[\"{}\"]\n", i, name));
    }

    // Generate edges
    for edge in graph.graph.edge_references() {
        let from_idx = edge.source().index();
        let to_idx = edge.target().index();
        output.push_str(&format!("    N{} --> N{}\n", from_idx, to_idx));
    }

    output.push_str("```\n");
    output
}

fn render_dot(graph: &DependencyGraph) -> String {
    let mut output = String::from("digraph dependencies {\n");
    output.push_str("    rankdir=LR;\n");
    output.push_str("    node [shape=box];\n\n");

    // Get all nodes
    let nodes: Vec<_> = graph
        .graph
        .node_indices()
        .map(|idx| &graph.graph[idx])
        .collect();

    // Node definitions
    for (i, node) in nodes.iter().enumerate() {
        let name = node
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        output.push_str(&format!("    N{} [label=\"{}\"];\n", i, name));
    }

    output.push('\n');

    // Edges
    for edge in graph.graph.edge_references() {
        let from_idx = edge.source().index();
        let to_idx = edge.target().index();
        output.push_str(&format!("    N{} -> N{};\n", from_idx, to_idx));
    }

    output.push_str("}\n");
    output
}

fn render_ascii(graph: &DependencyGraph) -> String {
    let stats = graph.stats();
    let mut output = format!(
        "Dependency Graph:\n  Nodes: {}\n  Edges: {}\n  Cycles: {}\n\n",
        stats.node_count,
        stats.edge_count,
        if stats.has_cycles { "Yes" } else { "No" }
    );

    // List nodes with their dependencies
    for node_idx in graph.graph.node_indices() {
        let node = &graph.graph[node_idx];
        let name = node
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let deps: Vec<_> = graph
            .graph
            .neighbors(node_idx)
            .map(|dep_idx| {
                graph.graph[dep_idx]
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
            })
            .collect();

        if deps.is_empty() {
            output.push_str(&format!("  {}\n", name));
        } else {
            output.push_str(&format!("  {} -> {}\n", name, deps.join(", ")));
        }
    }

    output
}

fn render_json(graph: &DependencyGraph) -> String {
    let nodes: Vec<_> = graph
        .graph
        .node_indices()
        .map(|idx| {
            let path = &graph.graph[idx];
            json!({
                "id": idx.index(),
                "path": path.to_string_lossy(),
                "name": path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
            })
        })
        .collect();

    let edges: Vec<_> = graph
        .graph
        .edge_references()
        .map(|edge| {
            json!({
                "from": edge.source().index(),
                "to": edge.target().index()
            })
        })
        .collect();

    let stats = graph.stats();

    let output = json!({
        "nodes": nodes,
        "edges": edges,
        "stats": {
            "node_count": stats.node_count,
            "edge_count": stats.edge_count,
            "has_cycles": stats.has_cycles
        }
    });

    serde_json::to_string_pretty(&output).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cue_common::Document;
    use std::path::PathBuf;

    fn create_doc(path: &str, links: Vec<String>) -> Document {
        Document {
            path: PathBuf::from(path),
            frontmatter: None,
            hash: "test".to_string(),
            tokens: 100,
            anchors: vec![],
            links,
        }
    }

    #[test]
    fn test_mermaid_format() {
        let docs = vec![
            create_doc("a.md", vec!["b.md".to_string()]),
            create_doc("b.md", vec![]),
        ];

        let graph = DependencyGraph::build(&docs).unwrap();
        let output = render(&graph, GraphFormat::Mermaid);

        assert!(output.contains("```mermaid"));
        assert!(output.contains("flowchart LR"));
        assert!(output.contains("a.md"));
        assert!(output.contains("b.md"));
        assert!(output.contains("-->"));
    }

    #[test]
    fn test_dot_format() {
        let docs = vec![
            create_doc("a.md", vec!["b.md".to_string()]),
            create_doc("b.md", vec![]),
        ];

        let graph = DependencyGraph::build(&docs).unwrap();
        let output = render(&graph, GraphFormat::Dot);

        assert!(output.contains("digraph dependencies"));
        assert!(output.contains("a.md"));
        assert!(output.contains("b.md"));
        assert!(output.contains("->"));
    }

    #[test]
    fn test_json_format() {
        let docs = vec![
            create_doc("a.md", vec!["b.md".to_string()]),
            create_doc("b.md", vec![]),
        ];

        let graph = DependencyGraph::build(&docs).unwrap();
        let output = render(&graph, GraphFormat::Json);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["nodes"].is_array());
        assert!(parsed["edges"].is_array());
        assert_eq!(parsed["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["edges"].as_array().unwrap().len(), 1);
    }
}
