//! Core engine for CueDeck
//!
//! This crate contains the core logic for parsing, graph resolution, and scene generation.

use cue_common::{Anchor, CueError, Document, Result};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub mod agent_fs;
pub mod batch;
pub mod cache;
pub mod code_search;
pub mod consistency;
pub mod context;
pub mod db;
pub mod doctor;
pub mod embedding_cache;
pub mod embeddings;
pub mod engine;
pub mod graph;
pub mod graph_viz;
pub mod query_lang;
pub mod task_filters;
pub mod task_graph;
pub mod tasks;

// Re-exports
pub use context::search_workspace;
pub use graph::DependencyGraph;
pub use task_filters::{TaskFilters, DateFilter, DateOperator, DateValue};
pub use query_lang::{ParsedQuery, QueryParser, FilterValue};
pub use batch::{BatchQuery, Query, BatchResponse, QueryResult, batch_query};

// Re-export commonly used functions
pub use context::save_embedding_cache;

/// Parse a markdown file into a Document
#[tracing::instrument(skip_all, fields(path = ?path))]
pub fn parse_file(path: &Path) -> Result<Document> {
    tracing::info!("Parsing file: {:?}", path);
    // Standardize path separators for consistency
    let path_str = path.to_string_lossy().replace('\\', "/");

    // Read file content
    let content = fs::read_to_string(path).map_err(|_| CueError::FileNotFound {
        path: path_str.clone(),
    })?;

    // Calculate SHA256 hash
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    // Regex for Frontmatter: ^---\n(content)\n---
    // Using simple string splitting for less dev-dependency overhead/complexity if possible,
    // but regex is robust for mixed line endings.
    // However, since we added `regex` crate:
    let frontmatter_regex = regex::Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---").unwrap();

    let (frontmatter, content_body) = if let Some(captures) = frontmatter_regex.captures(&content) {
        let yaml_str = captures.get(1).unwrap().as_str();
        match serde_yaml::from_str::<cue_common::CardMetadata>(yaml_str) {
            Ok(meta) => (Some(meta), &content[captures.get(0).unwrap().end()..]),
            Err(e) => {
                tracing::warn!("Failed to parse frontmatter in {:?}: {}", path, e);
                (None, content.as_str())
            }
        }
    } else {
        (None, content.as_str())
    };

    // Parse anchors (headings) with proper range calculation
    let mut anchors = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with('#') {
            let level = line.chars().take_while(|&c| c == '#').count() as u8;
            if (1..=6).contains(&level) {
                let header = line[level as usize..].trim().to_string();
                if !header.is_empty() {
                    // Create slug (lowercase, hyphens)
                    let slug = header
                        .to_lowercase()
                        .replace(" ", "-")
                        .chars()
                        .filter(|c| c.is_alphanumeric() || *c == '-')
                        .collect::<String>();

                    // Calculate end_line: extends to next heading line or EOF
                    let end_line = lines
                        .iter()
                        .enumerate()
                        .skip(i + 1)
                        .find(|(_, next_line)| next_line.starts_with('#'))
                        .map(|(next_i, _)| next_i + 1) // Convert 0-index to 1-indexed line number
                        .unwrap_or(lines.len());

                    anchors.push(Anchor {
                        slug,
                        header,
                        level,
                        start_line: i + 1,
                        end_line,
                    });
                }
            }
        }
    }

    // Link Extraction
    // Wiki-links: [[slug]]
    let wiki_link_regex = regex::Regex::new(r"\[\[(.*?)\]\]").unwrap();
    let mut links = Vec::new();

    for cap in wiki_link_regex.captures_iter(content_body) {
        if let Some(m) = cap.get(1) {
            links.push(m.as_str().trim().to_string());
        }
    }

    // Count tokens (heuristic: ~4 chars per token)
    let tokens = content.len() / 4;

    Ok(Document {
        path: PathBuf::from(path_str),
        frontmatter,
        hash,
        tokens,
        anchors,
        links,
    })
}

/// Resolve dependency graph
#[tracing::instrument(skip_all, fields(doc_count = docs.len()))]
pub fn resolve_graph(docs: &[Document]) -> Result<Vec<PathBuf>> {
    tracing::info!("Resolving graph for {} documents", docs.len());

    // Build adjacency list for dependency graph
    let mut graph: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut all_nodes: HashSet<PathBuf> = HashSet::new();

    // Create lookup maps for link resolution
    // map: slug -> path
    // map: filename -> path
    let mut slug_map: HashMap<String, PathBuf> = HashMap::new();
    let mut file_map: HashMap<String, PathBuf> = HashMap::new();

    for doc in docs {
        all_nodes.insert(doc.path.clone());
        graph.entry(doc.path.clone()).or_default();

        // Populate maps
        // 1. Filename (e.g. "task.md")
        if let Some(name) = doc.path.file_name().and_then(|n| n.to_str()) {
            file_map.insert(name.to_lowercase(), doc.path.clone());
            // Also without extension
            if let Some(stem) = doc.path.file_stem().and_then(|s| s.to_str()) {
                file_map.insert(stem.to_lowercase(), doc.path.clone());
            }
        }

        // 2. Slugs from anchors (only top-level or specific logic?)
        // For now, let's map the 'Title' slug from frontmatter if available, or first header
        if let Some(fm) = &doc.frontmatter {
            let slug = fm.title.to_lowercase().replace(" ", "-");
            slug_map.insert(slug, doc.path.clone());
        }
        // Also map anchors? (Might be too granular for file-dependency, but useful for context)
    }

    // Build edges
    for doc in docs {
        let dependencies = graph.entry(doc.path.clone()).or_default();

        for link in &doc.links {
            let link_lower = link.to_lowercase();

            // Try to resolve link
            let target = slug_map
                .get(&link_lower)
                .or_else(|| file_map.get(&link_lower));

            if let Some(target_path) = target {
                // Self-references don't count for cycle detection usually, but strictly they form a cycle of len 1.
                // We'll ignore self-loops for dependency resolution.
                if target_path != &doc.path && !dependencies.contains(target_path) {
                    dependencies.push(target_path.clone());
                }
            }
        }
    }

    // Topological sort using DFS
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    let mut rec_stack = HashSet::new();

    fn dfs(
        node: &PathBuf,
        graph: &HashMap<PathBuf, Vec<PathBuf>>,
        visited: &mut HashSet<PathBuf>,
        rec_stack: &mut HashSet<PathBuf>,
        result: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if rec_stack.contains(node) {
            // Cycle detected!
            // We could return the specific cycle path for better error,
            // but for now simple error.
            // Or we could log warning and ignore the back-edge to recover?
            // "Strict" mode usually fails.
            return Err(CueError::CycleDetected);
        }

        if visited.contains(node) {
            return Ok(());
        }

        rec_stack.insert(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                dfs(neighbor, graph, visited, rec_stack, result)?;
            }
        }

        rec_stack.remove(node);
        visited.insert(node.clone());
        result.push(node.clone());

        Ok(())
    }

    for node in all_nodes.iter() {
        if !visited.contains(node) {
            dfs(node, &graph, &mut visited, &mut rec_stack, &mut result)?;
        }
    }

    Ok(result)
}

/// Parse multiple files in parallel
///
/// # Arguments
/// * `paths` - Iterator of file paths to parse
///
/// # Returns
/// Vector of successfully parsed documents
pub fn parse_files_parallel<I>(paths: I) -> Vec<Document>
where
    I: IntoIterator<Item = PathBuf>,
    I::IntoIter: Send,
{
    let paths: Vec<PathBuf> = paths.into_iter().collect();

    paths
        .par_iter()
        .filter_map(|path| match parse_file(path) {
            Ok(doc) => Some(doc),
            Err(e) => {
                tracing::warn!("Failed to parse {:?}: {}", path, e);
                None
            }
        })
        .collect()
}

/// Generate scene content
#[tracing::instrument(skip(workspace_root), fields(workspace = ?workspace_root))]
pub fn generate_scene(workspace_root: &Path) -> Result<String> {
    use crate::engine::CueEngine;

    // Use the engine for Scene generation
    // This ensures consistent behavior between CLI one-off and watch mode
    let engine = CueEngine::new(workspace_root)?;
    engine.render()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_parse_file_basic() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.md");
        file.write_str("# Hello\n\nContent").unwrap();

        let doc = parse_file(file.path()).unwrap();
        assert_eq!(doc.anchors.len(), 1);
        assert_eq!(doc.anchors[0].header, "Hello");
        assert_eq!(doc.anchors[0].level, 1);
    }

    #[test]
    fn test_parse_file_with_frontmatter() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("card.md");
        file.write_str("---\ntitle: Test Card\ntype: feature\n---\n# Content\n\nBody")
            .unwrap();

        let doc = parse_file(file.path()).unwrap();
        assert!(doc.frontmatter.is_some());
        let fm = doc.frontmatter.unwrap();
        assert_eq!(fm.title, "Test Card");
    }

    #[test]
    fn test_parse_file_anchor_ranges() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.md");
        file.write_str("# First\nContent 1\n## Second\nContent 2\n### Third\nContent 3")
            .unwrap();

        let doc = parse_file(file.path()).unwrap();
        assert_eq!(doc.anchors.len(), 3);

        // First heading spans until second heading
        assert_eq!(doc.anchors[0].start_line, 1);
        assert_eq!(doc.anchors[0].end_line, 3);

        // Second heading spans until third
        assert_eq!(doc.anchors[1].start_line, 3);
        assert_eq!(doc.anchors[1].end_line, 5);

        // Third heading spans to EOF
        assert_eq!(doc.anchors[2].start_line, 5);
        assert_eq!(doc.anchors[2].end_line, 6);
    }

    #[test]
    fn test_parse_file_links() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.md");
        file.write_str("# Test\n\nSee [[other-doc]] and [[another]]")
            .unwrap();

        let doc = parse_file(file.path()).unwrap();
        assert_eq!(doc.links.len(), 2);
        assert!(doc.links.contains(&"other-doc".to_string()));
        assert!(doc.links.contains(&"another".to_string()));
    }

    #[test]
    fn test_resolve_graph_simple() {
        let temp = assert_fs::TempDir::new().unwrap();

        let a = temp.child("a.md");
        a.write_str("---\ntitle: A\ntype: feature\n---\n# A")
            .unwrap();

        let b = temp.child("b.md");
        b.write_str("---\ntitle: B\ntype: feature\n---\n# B\n\n[[A]]")
            .unwrap();

        let doc_a = parse_file(a.path()).unwrap();
        let doc_b = parse_file(b.path()).unwrap();

        let result = resolve_graph(&[doc_a.clone(), doc_b.clone()]).unwrap();

        // B depends on A, so A should come before B
        let a_pos = result.iter().position(|p| p == &doc_a.path).unwrap();
        let b_pos = result.iter().position(|p| p == &doc_b.path).unwrap();
        assert!(a_pos < b_pos);
    }

    #[test]
    fn test_resolve_graph_cycle_detection() {
        let temp = assert_fs::TempDir::new().unwrap();

        let a = temp.child("a.md");
        a.write_str("---\ntitle: A\ntype: feature\n---\n# A\n\n[[B]]")
            .unwrap();

        let b = temp.child("b.md");
        b.write_str("---\ntitle: B\ntype: feature\n---\n# B\n\n[[A]]")
            .unwrap();

        let doc_a = parse_file(a.path()).unwrap();
        let doc_b = parse_file(b.path()).unwrap();

        let result = resolve_graph(&[doc_a, doc_b]);
        assert!(result.is_err());
    }
}
