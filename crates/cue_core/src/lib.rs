//! Core engine for CueDeck
//!
//! This crate contains the core logic for parsing, graph resolution, and scene generation.

use cue_common::{Document, Anchor, Result, CueError};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use sha2::{Sha256, Digest};
use std::fs;

pub mod tasks;
pub mod context;
pub mod embeddings;

/// Parse a markdown file into a Document
#[tracing::instrument(skip_all, fields(path = ?path))]
pub fn parse_file(path: &Path) -> Result<Document> {
    tracing::info!("Parsing file: {:?}", path);
    // Standardize path separators for consistency
    let path_str = path.to_string_lossy().replace('\\', "/");
    
    // Read file content
    let content = fs::read_to_string(path)
        .map_err(|_| CueError::FileNotFound { 
            path: path_str.clone() 
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
    
    // Parse anchors (headings) using manual check (efficient enough)
    let mut anchors = Vec::new();    // Start line parsing
    let _current_line = 1;
    // Adjust current_line if frontmatter was stripped?
    // Actually, we should parse lines from original content to keep line numbers accurate.
    
    for (i, line) in content.lines().enumerate() {
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
                    
                    anchors.push(Anchor {
                        slug,
                        header,
                        level,
                        start_line: i + 1,
                        end_line: i + 1, // TODO: calculate proper range
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
            let target = slug_map.get(&link_lower)
                .or_else(|| file_map.get(&link_lower));
                
            if let Some(target_path) = target {
                // Self-references don't count for cycle detection usually, but strictly they form a cycle of len 1.
                // We'll ignore self-loops for dependency resolution.
                if target_path != &doc.path {
                    if !dependencies.contains(target_path) {
                        dependencies.push(target_path.clone());
                    }
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

/// Generate scene content
#[tracing::instrument(skip(workspace_root), fields(workspace = ?workspace_root))]
pub fn generate_scene(workspace_root: &Path) -> Result<String> {
    tracing::info!("Generating scene for workspace: {:?}", workspace_root);
    
    // Load config
    let config = cue_config::Config::load(workspace_root)?;
    
    // Find all markdown files in cards/ and docs/
    let cards_dir = workspace_root.join(".cuedeck/cards");
    let docs_dir = workspace_root.join(".cuedeck/docs");
    
    let mut documents = Vec::new();
    
    // Parse cards if directory exists
    if cards_dir.exists() {
        for entry in walkdir::WalkDir::new(&cards_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
                match parse_file(entry.path()) {
                    Ok(doc) => documents.push(doc),
                    Err(e) => tracing::warn!("Failed to parse {:?}: {}", entry.path(), e),
                }
            }
        }
    }
    
    // Parse docs if directory exists
    if docs_dir.exists() {
        for entry in walkdir::WalkDir::new(&docs_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
                match parse_file(entry.path()) {
                    Ok(doc) => documents.push(doc),
                    Err(e) => tracing::warn!("Failed to parse {:?}: {}", entry.path(), e),
                }
            }
        }
    }
    
    // Resolve dependencies
    let ordered = resolve_graph(&documents)?;
    
    // Generate scene markdown
    let mut scene = String::from("# Scene Context\n\n");
    scene.push_str(&format!("Generated: {}\n", chrono::Utc::now().to_rfc3339()));
    scene.push_str(&format!("Documents: {}\n\n", documents.len()));
    
    let mut total_tokens = 0;
    
    for path in ordered.iter() {
        if let Some(doc) = documents.iter().find(|d| &d.path == path) {
            if total_tokens + doc.tokens > config.budgets.feature {
                tracing::warn!("Token limit reached, truncating scene");
                break;
            }
            
            scene.push_str(&format!("## {}\n\n", path.display()));
            scene.push_str(&format!("Tokens: {} | Hash: {}\n\n", doc.tokens, &doc.hash[..8]));
            
            // Add anchors
            if !doc.anchors.is_empty() {
                scene.push_str("### Anchors\n\n");
                for anchor in &doc.anchors {
                    scene.push_str(&format!("- {} (L{}): {}\n", 
                        "#".repeat(anchor.level as usize),
                        anchor.start_line,
                        anchor.header
                    ));
                }
                scene.push('\n');
            }
            
            total_tokens += doc.tokens;
        }
    }
    
    scene.push_str(&format!("\n---\nTotal Tokens: {}\n", total_tokens));
    
    Ok(scene)
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused
    // use std::path::PathBuf; // Unused 

    // TODO: Add real tests with temporary files
}
