//! Task dependency graph management
//!
//! This module provides task-specific dependency tracking and validation.

use crate::{CueError, Result};
use cue_common::TaskDependency;
use petgraph::algo::is_cyclic_directed;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::path::Path;

/// Task dependency graph for tracking task relationships
pub struct TaskGraph {
    graph: DiGraph<String, ()>, // Nodes are task IDs
    task_to_node: HashMap<String, NodeIndex>,
}

/// Comprehensive graph statistics
#[derive(Debug, Clone)]
pub struct GraphStats {
    pub total_tasks: usize,
    pub total_dependencies: usize,
    pub orphaned_tasks: usize,
    pub tasks_with_dependencies: usize,
    pub max_dependency_depth: usize,
}


impl TaskGraph {
    /// Create a new empty task graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            task_to_node: HashMap::new(),
        }
    }

    /// Build task graph from workspace by scanning all task cards
    #[tracing::instrument(skip_all, fields(workspace = ?workspace_root))]
    pub fn from_workspace(workspace_root: &Path) -> Result<Self> {
        use std::fs;
        use walkdir::WalkDir;

        let mut graph = Self::new();
        let cards_dir = workspace_root.join(".cuedeck/cards");

        if !cards_dir.exists() {
            return Ok(graph);
        }

        // First pass: add all task nodes
        for entry in WalkDir::new(&cards_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md")
            {
                // Extract task ID from filename (e.g., "abc123.md" -> "abc123")
                if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    graph.add_task(stem);
                }
            }
        }

        // Second pass: add dependency edges by parsing frontmatter
        for entry in WalkDir::new(&cards_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md")
            {
                if let Some(task_id) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    // Parse frontmatter to extract depends_on
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Some(deps) = extract_depends_on(&content) {
                            for dep_id in deps {
                                // Ignore errors if dependency doesn't exist (validation will catch this later)
                                let _ = graph.add_dependency(task_id, &dep_id);
                            }
                        }
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Add a task to the graph
    pub fn add_task(&mut self, task_id: &str) -> NodeIndex {
        *self
            .task_to_node
            .entry(task_id.to_string())
            .or_insert_with(|| self.graph.add_node(task_id.to_string()))
    }

    /// Add a dependency edge (from depends on to)
    pub fn add_dependency(&mut self, from: &str, to: &str) -> Result<()> {
        // Ensure both nodes exist
        let from_node = self.add_task(from);
        let to_node = self.add_task(to);

        // Add edge: from -> to means "from depends on to"
        self.graph.add_edge(from_node, to_node, ());

        Ok(())
    }

    /// Validate that no circular dependencies exist
    pub fn validate_dependencies(&self) -> Result<()> {
        if is_cyclic_directed(&self.graph) {
            // Find the cycle path for better error message
            if let Some(cycle) = self.find_cycle() {
                let cycle_str = cycle.join(" -> ");
                return Err(CueError::CircularDependency(cycle_str));
            }
            return Err(CueError::CycleDetected);
        }
        Ok(())
    }

    /// Get all direct dependencies for a task (tasks that this task depends on)
    pub fn get_dependencies(&self, task_id: &str) -> Vec<String> {
        if let Some(&node) = self.task_to_node.get(task_id) {
            self.graph
                .neighbors(node)
                .map(|n| self.graph[n].clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all dependents (tasks that depend on this task)
    pub fn get_dependents(&self, task_id: &str) -> Vec<String> {
        if let Some(&node) = self.task_to_node.get(task_id) {
            self.graph
                .neighbors_directed(node, petgraph::Direction::Incoming)
                .map(|n| self.graph[n].clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if adding a dependency would create a cycle
    pub fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        // Clone the graph to test
        let mut test_graph = self.graph.clone();
        let test_task_to_node = self.task_to_node.clone();

        // Get or simulate nodes
        let from_node = if let Some(&node) = test_task_to_node.get(from) {
            node
        } else {
            test_graph.add_node(from.to_string())
        };

        let to_node = if let Some(&node) = test_task_to_node.get(to) {
            node
        } else {
            test_graph.add_node(to.to_string())
        };

        // Try adding the edge
        test_graph.add_edge(from_node, to_node, ());

        // Check if cyclic
        is_cyclic_directed(&test_graph)
    }

    /// Find a cycle in the graph (for error reporting)
    fn find_cycle(&self) -> Option<Vec<String>> {
        use petgraph::visit::depth_first_search;
        use petgraph::visit::DfsEvent;
        use std::collections::HashSet;

        let mut cycle_path: Vec<String> = Vec::new();
        let mut in_stack: HashSet<NodeIndex> = HashSet::new();
        let mut found_cycle = false;

        depth_first_search(&self.graph, self.graph.node_indices(), |event| {
            match event {
                DfsEvent::Discover(n, _) => {
                    in_stack.insert(n);
                    cycle_path.push(self.graph[n].clone());
                }
                DfsEvent::Finish(n, _) => {
                    in_stack.remove(&n);
                    if !found_cycle {
                        cycle_path.pop();
                    }
                }
                DfsEvent::BackEdge(_, target) => {
                    // Found a back edge - this indicates a cycle
                    if in_stack.contains(&target) {
                        // Complete the cycle by  adding the target
                        cycle_path.push(self.graph[target].clone());
                        found_cycle = true;
                        return petgraph::visit::Control::Break(());
                    }
                }
                _ => {}
            }
            petgraph::visit::Control::Continue
        });

        if found_cycle {
            Some(cycle_path)
        } else {
            None
        }
    }

    /// Get all task dependencies as a list
    pub fn get_all_dependencies(&self) -> Vec<TaskDependency> {
        self.graph
            .edge_indices()
            .filter_map(|edge| {
                let (from_node, to_node) = self.graph.edge_endpoints(edge)?;
                Some(TaskDependency {
                    from_id: self.graph[from_node].clone(),
                    to_id: self.graph[to_node].clone(),
                })
            })
            .collect()
    }

    /// Find tasks with no dependencies and no dependents (isolated tasks)
    pub fn find_orphaned_tasks(&self) -> Vec<String> {
        self.task_to_node
            .iter()
            .filter_map(|(task_id, &node)| {
                let has_dependencies = self.graph.neighbors(node).count() > 0;
                let has_dependents = self
                    .graph
                    .neighbors_directed(node, petgraph::Direction::Incoming)
                    .count()
                    > 0;

                if !has_dependencies && !has_dependents {
                    Some(task_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if any task references non-existent dependencies
    /// Returns: Vec<(task_id, missing_dep_id)>
    pub fn check_missing_dependencies(&self, workspace_root: &Path) -> Vec<(String, String)> {
        use std::fs;

        let mut missing = Vec::new();
        let cards_dir = workspace_root.join(".cuedeck/cards");

        for (task_id, _) in &self.task_to_node {
            let task_path = cards_dir.join(format!("{}.md", task_id));
            if let Ok(content) = fs::read_to_string(&task_path) {
                if let Some(deps) = extract_depends_on(&content) {
                    for dep_id in deps {
                        if !self.task_to_node.contains_key(&dep_id) {
                            missing.push((task_id.clone(), dep_id));
                        }
                    }
                }
            }
        }

        missing
    }

    /// Get comprehensive graph statistics
    pub fn get_graph_stats(&self) -> GraphStats {
        use petgraph::algo::dijkstra;

        let orphaned = self.find_orphaned_tasks();
        let tasks_with_deps = self
            .task_to_node
            .iter()
            .filter(|(_, &node)| self.graph.neighbors(node).count() > 0)
            .count();

        // Calculate max depth using longest path
        let max_depth = self
            .task_to_node
            .values()
            .map(|&node| {
                let distances = dijkstra(&self.graph, node, None, |_| 1);
                distances.values().max().copied().unwrap_or(0)
            })
            .max()
            .unwrap_or(0);

        GraphStats {
            total_tasks: self.task_to_node.len(),
            total_dependencies: self.graph.edge_count(),
            orphaned_tasks: orphaned.len(),
            tasks_with_dependencies: tasks_with_deps,
            max_dependency_depth: max_depth,
        }
    }

    /// Export graph as DOT format (for Graphviz)
    pub fn to_dot(&self) -> String {
        use std::fmt::Write;

        let mut output = String::from("digraph TaskGraph {\n");
        output.push_str("  node [shape=box, style=rounded];\n");
        output.push_str("  rankdir=LR;\n");

        // Add nodes
        for (task_id, _) in &self.task_to_node {
            writeln!(&mut output, "  \"{}\"", task_id).unwrap();
        }

        // Add edges
        for edge in self.graph.edge_references() {
            let from = &self.graph[edge.source()];
            let to = &self.graph[edge.target()];
            writeln!(&mut output, "  \"{}\" -> \"{}\"", from, to).unwrap();
        }

        output.push_str("}\n");
        output
    }

    /// Export graph as Mermaid flowchart
    pub fn to_mermaid(&self) -> String {
        use std::fmt::Write;

        let mut output = String::from("graph LR\n");

        // Add edges (nodes are implicit in Mermaid)
        for edge in self.graph.edge_references() {
            let from = &self.graph[edge.source()];
            let to = &self.graph[edge.target()];
            // Use sanitized IDs for Mermaid node names
            writeln!(&mut output, "  {}[\"{}\"] --> {}[\"{}\"]", 
                     sanitize_mermaid_id(from), from,
                     sanitize_mermaid_id(to), to).unwrap();
        }

        // If no edges, show all nodes standalone
        if self.graph.edge_count() == 0 {
            for (task_id, _) in &self.task_to_node {
                writeln!(&mut output, "  {}[\"{}\"]", 
                         sanitize_mermaid_id(task_id), task_id).unwrap();
            }
        }

        output
    }

    /// Export graph as JSON
    pub fn to_json(&self) -> Result<String> {
        use serde_json::json;

        let nodes: Vec<_> = self.task_to_node.keys().cloned().collect();
        let edges: Vec<_> = self
            .graph
            .edge_references()
            .map(|e| {
                json!({
                    "from": self.graph[e.source()],
                    "to": self.graph[e.target()]
                })
            })
            .collect();

        let graph_json = json!({
            "nodes": nodes,
            "edges": edges
        });

        Ok(serde_json::to_string_pretty(&graph_json)?)
    }
}

/// Sanitize task ID for use in Mermaid diagrams
fn sanitize_mermaid_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to extract depends_on field from markdown frontmatter
fn extract_depends_on(content: &str) -> Option<Vec<String>> {
    use regex::Regex;

    let frontmatter_regex = Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---").ok()?;
    let captures = frontmatter_regex.captures(content)?;
    let yaml_str = captures.get(1)?.as_str();

    // Parse YAML
    let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).ok()?;

    // Extract depends_on field
    yaml.get("depends_on")?
        .as_sequence()?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect::<Vec<_>>()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = TaskGraph::new();
        assert_eq!(graph.get_dependencies("task1").len(), 0);
    }

    #[test]
    fn test_add_task() {
        let mut graph = TaskGraph::new();
        graph.add_task("task1");
        graph.add_task("task2");

        assert_eq!(graph.task_to_node.len(), 2);
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = TaskGraph::new();
        graph.add_dependency("task1", "task2").unwrap();

        let deps = graph.get_dependencies("task1");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "task2");
    }

    #[test]
    fn test_circular_dependency() {
        let mut graph = TaskGraph::new();
        graph.add_dependency("task1", "task2").unwrap();
        graph.add_dependency("task2", "task3").unwrap();
        graph.add_dependency("task3", "task1").unwrap();

        let result = graph.validate_dependencies();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CueError::CircularDependency(_)));
    }

    #[test]
    fn test_would_create_cycle() {
        let mut graph = TaskGraph::new();
        graph.add_dependency("task1", "task2").unwrap();
        graph.add_dependency("task2", "task3").unwrap();

        // This would create a cycle
        assert!(graph.would_create_cycle("task3", "task1"));

        // This would not
        assert!(!graph.would_create_cycle("task1", "task4"));
    }

    #[test]
    fn test_get_dependents() {
        let mut graph = TaskGraph::new();
        graph.add_dependency("task1", "task2").unwrap();
        graph.add_dependency("task3", "task2").unwrap();

        let dependents = graph.get_dependents("task2");
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"task1".to_string()));
        assert!(dependents.contains(&"task3".to_string()));
    }

    #[test]
    fn test_find_orphaned_tasks() {
        let mut graph = TaskGraph::new();
        
        // Add connected tasks
        graph.add_dependency("task1", "task2").unwrap();
        graph.add_dependency("task2", "task3").unwrap();
        
        // Add orphaned tasks (no dependencies, no dependents)
        graph.add_task("orphan1");
        graph.add_task("orphan2");
        
        let orphaned = graph.find_orphaned_tasks();
        assert_eq!(orphaned.len(), 2);
        assert!(orphaned.contains(&"orphan1".to_string()));
        assert!(orphaned.contains(&"orphan2".to_string()));
        
        // Connected tasks should not be orphaned
        assert!(!orphaned.contains(&"task1".to_string()));
        assert!(!orphaned.contains(&"task2".to_string()));
        assert!(!orphaned.contains(&"task3".to_string()));
    }

    #[test]
    fn test_get_graph_stats() {
        let mut graph = TaskGraph::new();
        
        // Empty graph stats
        let stats = graph.get_graph_stats();
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.total_dependencies, 0);
        assert_eq!(stats.orphaned_tasks, 0);
        
        // Build a test graph:
        // task1 -> task2 -> task3
        // task4 (orphaned)
        graph.add_dependency("task1", "task2").unwrap();
        graph.add_dependency("task2", "task3").unwrap();
        graph.add_task("task4");
        
        let stats = graph.get_graph_stats();
        assert_eq!(stats.total_tasks, 4);
        assert_eq!(stats.total_dependencies, 2);
        assert_eq!(stats.orphaned_tasks, 1);
        assert_eq!(stats.tasks_with_dependencies, 2); // task1 and task2
        assert_eq!(stats.max_dependency_depth, 2);
    }
}
