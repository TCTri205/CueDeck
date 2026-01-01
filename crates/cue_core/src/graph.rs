//! Dependency graph resolution and analysis
//!
//! This module provides robust graph algorithms for:
//! - Cycle detection
//! - Topological sorting
//! - Dependency analysis

use cue_common::{CueError, Document, Result};
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::path::PathBuf;

/// Dependency graph built from document links
pub struct DependencyGraph {
    pub(crate) graph: DiGraph<PathBuf, ()>,
    path_to_node: HashMap<PathBuf, NodeIndex>,
    // Reverse index for efficient lookups
    slug_map: HashMap<String, PathBuf>,
    file_map: HashMap<String, PathBuf>,
}

impl DependencyGraph {
    /// Build a dependency graph from documents
    ///
    /// Documents are nodes, and links are directed edges.
    /// Edge A -> B means "A depends on B" or "A links to B".
    #[tracing::instrument(skip_all, fields(doc_count = docs.len()))]
    pub fn build(docs: &[Document]) -> Result<Self> {
        tracing::info!("Building dependency graph from {} documents", docs.len());

        let mut graph = DiGraph::new();
        let mut path_to_node = HashMap::new();

        // Step 1: Create nodes for all documents
        for doc in docs {
            let node = graph.add_node(doc.path.clone());
            path_to_node.insert(doc.path.clone(), node);
        }

        // Step 2: Create lookup maps for link resolution
        let mut slug_map: HashMap<String, PathBuf> = HashMap::new();
        let mut file_map: HashMap<String, PathBuf> = HashMap::new();

        for doc in docs {
            // Map filename (task.md, TASK.md -> task.md)
            if let Some(name) = doc.path.file_name().and_then(|n| n.to_str()) {
                file_map.insert(name.to_lowercase(), doc.path.clone());

                // Also map without extension (task.md -> task)
                if let Some(stem) = doc.path.file_stem().and_then(|s| s.to_str()) {
                    file_map.insert(stem.to_lowercase(), doc.path.clone());
                }
            }

            // Map slugs from frontmatter title
            if let Some(fm) = &doc.frontmatter {
                let slug = fm.title.to_lowercase().replace(' ', "-");
                slug_map.insert(slug, doc.path.clone());
            }
        }

        // Step 3: Add edges based on document links
        for doc in docs {
            let from_node = path_to_node[&doc.path];

            for link in &doc.links {
                // Try to resolve link target
                let target_path = if link.starts_with("./") || link.starts_with("../") {
                    // Relative path
                    doc.path
                        .parent()
                        .and_then(|parent| parent.join(link).canonicalize().ok())
                } else if link.contains('/') {
                    // Absolute or rooted path
                    Some(PathBuf::from(link))
                } else {
                    // Try filename or slug resolution
                    file_map
                        .get(&link.to_lowercase())
                        .cloned()
                        .or_else(|| slug_map.get(&link.to_lowercase()).cloned())
                };

                // Add edge if target exists in graph
                if let Some(target) = target_path {
                    if let Some(&to_node) = path_to_node.get(&target) {
                        graph.add_edge(from_node, to_node, ());
                        tracing::debug!(
                            "Added edge: {:?} -> {:?}",
                            doc.path.file_name().unwrap_or_default(),
                            target.file_name().unwrap_or_default()
                        );
                    }
                }
            }
        }

        Ok(DependencyGraph {
            graph,
            path_to_node,
            slug_map,
            file_map,
        })
    }

    /// Add or update a single document in the graph
    ///
    /// This is optimized for incremental updates (watch mode).
    /// Removes old edges from this document and recreates them based on current links.
    #[tracing::instrument(skip(self, doc), fields(path = ?doc.path))]
    pub fn add_or_update_document(&mut self, doc: &Document) {
        tracing::debug!("Updating document in graph: {:?}", doc.path);

        // Get or create node for this document
        let node = *self
            .path_to_node
            .entry(doc.path.clone())
            .or_insert_with(|| self.graph.add_node(doc.path.clone()));

        // Update index maps
        if let Some(name) = doc.path.file_name().and_then(|n| n.to_str()) {
            self.file_map.insert(name.to_lowercase(), doc.path.clone());
            if let Some(stem) = doc.path.file_stem().and_then(|s| s.to_str()) {
                self.file_map.insert(stem.to_lowercase(), doc.path.clone());
            }
        }
        if let Some(fm) = &doc.frontmatter {
            let slug = fm.title.to_lowercase().replace(' ', "-");
            self.slug_map.insert(slug, doc.path.clone());
        }

        // Remove all outgoing edges from this node
        let edges_to_remove: Vec<_> = self
            .graph
            .edges_directed(node, petgraph::Direction::Outgoing)
            .map(|e| e.id())
            .collect();

        for edge in edges_to_remove {
            self.graph.remove_edge(edge);
        }

        // Add new edges based on current links
        for link in &doc.links {
            let target_path = if link.starts_with("./") || link.starts_with("../") {
                doc.path
                    .parent()
                    .and_then(|parent| parent.join(link).canonicalize().ok())
            } else if link.contains('/') {
                Some(PathBuf::from(link))
            } else {
                self.file_map
                    .get(&link.to_lowercase())
                    .cloned()
                    .or_else(|| self.slug_map.get(&link.to_lowercase()).cloned())
            };

            if let Some(target) = target_path {
                if let Some(&to_node) = self.path_to_node.get(&target) {
                    self.graph.add_edge(node, to_node, ());
                }
            }
        }
    }

    /// Remove a document from the graph
    ///
    /// Optimized for incremental updates when files are deleted.
    #[tracing::instrument(skip(self), fields(path = ?path))]
    pub fn remove_document(&mut self, path: &PathBuf) {
        tracing::debug!("Removing document from graph: {:?}", path);

        if let Some(node) = self.path_to_node.remove(path) {
            self.graph.remove_node(node);

            // Clean up index maps
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                self.file_map.remove(&name.to_lowercase());
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    self.file_map.remove(&stem.to_lowercase());
                }
            }
            // Note: slug_map cleanup would require re-reading doc frontmatter,
            // so we skip it (stale entries are harmless)
        }
    }

    /// Detect if the graph contains cycles
    ///
    /// Returns `None` if no cycles, or `Some(cycle_path)` with the detected cycle.
    pub fn detect_cycle(&self) -> Option<Vec<PathBuf>> {
        if !is_cyclic_directed(&self.graph) {
            return None;
        }

        // Find an actual cycle path
        // We'll do a DFS and track the path
        for start_node in self.graph.node_indices() {
            if let Some(cycle) = self.find_cycle_from(start_node) {
                return Some(cycle);
            }
        }

        None
    }

    /// Find a cycle starting from a specific node
    fn find_cycle_from(&self, start: NodeIndex) -> Option<Vec<PathBuf>> {
        let mut visited = HashMap::new();
        let mut path_stack = Vec::new();

        if self.dfs_cycle(start, &mut visited, &mut path_stack) {
            Some(path_stack)
        } else {
            None
        }
    }

    /// DFS helper for cycle detection with path tracking
    fn dfs_cycle(
        &self,
        node: NodeIndex,
        visited: &mut HashMap<NodeIndex, VisitState>,
        path_stack: &mut Vec<PathBuf>,
    ) -> bool {
        match visited.get(&node) {
            Some(VisitState::InProgress) => {
                // Found a back edge - cycle detected
                // Add current node to complete the cycle
                path_stack.push(self.graph[node].clone());
                return true;
            }
            Some(VisitState::Done) => return false,
            None => {}
        }

        visited.insert(node, VisitState::InProgress);
        path_stack.push(self.graph[node].clone());

        for neighbor in self.graph.neighbors(node) {
            if self.dfs_cycle(neighbor, visited, path_stack) {
                return true;
            }
        }

        path_stack.pop();
        visited.insert(node, VisitState::Done);
        false
    }

    /// Sort documents in topological order
    ///
    /// Returns documents sorted such that dependencies come before dependents.
    /// For example, if A links to B, then B will appear before A in the result.
    /// Returns error if graph contains cycles.
    pub fn sort_topological(&self) -> Result<Vec<PathBuf>> {
        match toposort(&self.graph, None) {
            Ok(sorted_nodes) => {
                // Reverse because we want dependencies first
                // petgraph's toposort returns source-before-target order
                // but we semantically want "if A links to B, B comes first"
                let mut result: Vec<PathBuf> = sorted_nodes
                    .into_iter()
                    .map(|node| self.graph[node].clone())
                    .collect();
                result.reverse();
                Ok(result)
            }
            Err(_) => Err(CueError::CycleDetected),
        }
    }

    /// Find orphan documents (no incoming edges)
    pub fn orphans(&self) -> Vec<PathBuf> {
        self.graph
            .node_indices()
            .filter(|&node| {
                self.graph
                    .neighbors_directed(node, petgraph::Direction::Incoming)
                    .count()
                    == 0
            })
            .map(|node| self.graph[node].clone())
            .collect()
    }

    /// Get graph statistics
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.graph.node_count(),
            edge_count: self.graph.edge_count(),
            has_cycles: is_cyclic_directed(&self.graph),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    InProgress,
    Done,
}

#[derive(Debug, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub has_cycles: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cue_common::Document;

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
    fn test_empty_graph() {
        let docs = vec![];
        let graph = DependencyGraph::build(&docs).unwrap();

        assert_eq!(graph.stats().node_count, 0);
        assert_eq!(graph.stats().edge_count, 0);
    }

    #[test]
    fn test_acyclic_graph() {
        let docs = vec![
            create_doc("a.md", vec!["b.md".to_string()]),
            create_doc("b.md", vec!["c.md".to_string()]),
            create_doc("c.md", vec![]),
        ];

        let graph = DependencyGraph::build(&docs).unwrap();

        assert_eq!(graph.stats().node_count, 3);
        assert_eq!(graph.stats().edge_count, 2);
        assert!(!graph.stats().has_cycles);
        assert!(graph.detect_cycle().is_none());

        let sorted = graph.sort_topological().unwrap();
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    fn test_cycle_detection() {
        let docs = vec![
            create_doc("a.md", vec!["b.md".to_string()]),
            create_doc("b.md", vec!["c.md".to_string()]),
            create_doc("c.md", vec!["a.md".to_string()]), // Cycle: a -> b -> c -> a
        ];

        let graph = DependencyGraph::build(&docs).unwrap();

        assert!(graph.stats().has_cycles);

        let cycle = graph.detect_cycle();
        assert!(cycle.is_some(), "Should detect cycle");

        let cycle_path = cycle.unwrap();
        assert!(cycle_path.len() >= 3, "Cycle should have at least 3 nodes");

        // Topological sort should fail
        assert!(graph.sort_topological().is_err());
    }

    #[test]
    fn test_orphans() {
        let docs = vec![
            create_doc("a.md", vec!["b.md".to_string()]),
            create_doc("b.md", vec![]),
            create_doc("orphan.md", vec![]),
        ];

        let graph = DependencyGraph::build(&docs).unwrap();
        let orphans = graph.orphans();

        // Both a.md and orphan.md have no incoming edges
        assert_eq!(orphans.len(), 2);
    }
}
