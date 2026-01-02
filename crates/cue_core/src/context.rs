use crate::embedding_cache::EmbeddingCache;
use crate::embeddings::EmbeddingModel;
use crate::parse_file;
use cue_common::{Document, Result};
use lazy_static::lazy_static;
use std::cmp::Ordering;
use std::path::Path;
use std::sync::Mutex;
use walkdir::WalkDir;

lazy_static! {
    static ref EMBEDDING_CACHE: Mutex<EmbeddingCache> = {
        // Use current_dir as workspace (will be proper in full implementation)
        let workspace = std::env::current_dir().unwrap_or_default();
        let mut cache = EmbeddingCache::new(&workspace, 1000).expect("Failed to create embedding cache");
        
        // Try to load existing cache
        let _ = cache.load(); // Ignore errors, start fresh if corrupted
        
        Mutex::new(cache)
    };
}

/// Search mode selection
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchMode {
    /// Fast exact/fuzzy keyword matching
    Keyword,
    /// Semantic similarity using embeddings
    Semantic,
    /// Combined ranking (default): 70% semantic + 30% keyword
    #[default]
    Hybrid,
}

impl SearchMode {
    /// Parse from string (for CLI/MCP)
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "keyword" => Self::Keyword,
            "semantic" => Self::Semantic,
            _ => Self::Hybrid,  // Default to hybrid for unknown values
        }
    }
}

/// Configuration for hybrid search scoring
#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    /// Weight for semantic score (0.0-1.0)
    pub semantic_weight: f32,
    /// Weight for keyword score (0.0-1.0)
    pub keyword_weight: f32,
    /// Max raw keyword score for normalization
    pub keyword_max_score: i32,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            semantic_weight: 0.7,
            keyword_weight: 0.3,
            keyword_max_score: 200,
        }
    }
}

/// Optional filters for search results
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    /// Filter by tags (if document frontmatter has matching tags)
    pub tags: Option<Vec<String>>,
    /// Filter by priority
    pub priority: Option<String>,
}

/// Main search function with mode selection
#[tracing::instrument(skip(root))]
pub fn search_workspace_with_mode(
    root: &Path,
    query: &str,
    mode: SearchMode,
    _filters: Option<SearchFilters>,
) -> Result<Vec<Document>> {
    tracing::info!(
        "Searching workspace: {:?} for '{}' (mode: {:?})",
        root,
        query,
        mode
    );

    match mode {
        SearchMode::Keyword => search_workspace_keyword(root, query),
        SearchMode::Semantic => search_workspace_semantic(root, query, 10),
        SearchMode::Hybrid => search_workspace_hybrid(root, query),
    }
}

/// Backward-compatible search function (legacy API)
#[tracing::instrument(skip(root))]
pub fn search_workspace(root: &Path, query: &str, semantic: bool) -> Result<Vec<Document>> {
    let mode = if semantic {
        SearchMode::Semantic
    } else {
        SearchMode::Keyword
    };
    search_workspace_with_mode(root, query, mode, None)
}

/// Keyword-based search (original implementation)
fn search_workspace_keyword(root: &Path, query: &str) -> Result<Vec<Document>> {
    let query_lower = query.to_lowercase();
    let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();

    let mut results: Vec<(Document, i32)> = Vec::new();

    // Use walkdir to traverse
    // Filter ignored directories/files
    let walker = WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_ignored(e));

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    let path = entry.path();
                    // Calculate score
                    // We read the file content here for scoring content matches
                    let score = score_file(path, &query_lower, &query_tokens);

                    if score > 0 {
                        match parse_file(path) {
                            Ok(doc) => results.push((doc, score)),
                            Err(e) => tracing::warn!("Failed to parse {:?}: {}", path, e),
                        }
                    }
                }
            }
        }
    }

    // Sort by score descending
    results.sort_by(|a, b| b.1.cmp(&a.1));

    // Return top 10
    let top_docs = results.into_iter().take(10).map(|(doc, _)| doc).collect();

    Ok(top_docs)
}

/// Semantic search using vector embeddings
fn search_workspace_semantic(root: &Path, query: &str, limit: usize) -> Result<Vec<Document>> {
    use rayon::prelude::*;

    tracing::info!("Performing semantic search for: '{}'", query);

    // Generate query embedding (NOT cached - queries are one-shot)
    let query_embedding =
        EmbeddingModel::embed(query).map_err(|e| std::io::Error::other(e.to_string()))?;

    // Collect all markdown files first
    let mut md_files = Vec::new();
    let walker = WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_ignored(e));

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    md_files.push(entry.path().to_path_buf());
                }
            }
        }
    }

    // Parallel processing with cache
    let candidates: Vec<(Document, f32)> = md_files
        .par_iter()
        .filter_map(|path| {
            match parse_file(path) {
                Ok(doc) => {
                    // Read file content for embedding
                    if let Ok(content) = std::fs::read_to_string(&doc.path) {
                        // Get cached embedding or compute
                        let doc_embedding = {
                            let mut cache = EMBEDDING_CACHE.lock().unwrap();
                            cache.get_or_compute(&doc.hash, &content)
                        };
                        
                        match doc_embedding {
                            Ok(embedding) => {
                                let similarity = EmbeddingModel::cosine_similarity(
                                    &query_embedding,
                                    &embedding,
                                );
                                Some((doc, similarity))
                            }
                            Err(e) => {
                                tracing::warn!("Failed to get embedding for {:?}: {}", path, e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse {:?}: {}", path, e);
                    None
                }
            }
        })
        .collect();

    // Sort by similarity descending
    let mut sorted_candidates = candidates;
    sorted_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Return top N
    Ok(sorted_candidates
        .into_iter()
        .take(limit)
        .map(|(doc, _)| doc)
        .collect())
}

/// Normalize keyword score to [0, 1] range
fn normalize_keyword_score(raw_score: i32, max_score: i32) -> f32 {
    if max_score <= 0 {
        return 0.0;
    }
    (raw_score as f32 / max_score as f32).clamp(0.0, 1.0)
}

/// Hybrid search: combines keyword and semantic search with weighted scoring
fn search_workspace_hybrid(root: &Path, query: &str) -> Result<Vec<Document>> {
    use rayon::prelude::*;

    tracing::info!("Performing hybrid search for: '{}'", query);
    
    let config = HybridSearchConfig::default();
    let query_lower = query.to_lowercase();
    let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();

    // Collect all markdown files
    let mut md_files = Vec::new();
    let walker = WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_ignored(e));

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    md_files.push(entry.path().to_path_buf());
                }
            }
        }
    }

    // Generate query embedding for semantic search (NOT cached)
    let query_embedding = EmbeddingModel::embed(query)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Parallel processing: compute both keyword AND semantic scores with cache
    let candidates: Vec<(Document, i32, f32)> = md_files
        .par_iter()
        .filter_map(|path| {
            match parse_file(path) {
                Ok(doc) => {
                    // Compute keyword score
                    let keyword_score = score_file(path, &query_lower, &query_tokens);
                    
                    // Compute semantic score with cache
                    let semantic_score = if let Ok(content) = std::fs::read_to_string(&doc.path) {
                        let mut cache = EMBEDDING_CACHE.lock().unwrap();
                        match cache.get_or_compute(&doc.hash, &content) {
                            Ok(doc_embedding) => {
                                EmbeddingModel::cosine_similarity(&query_embedding, &doc_embedding)
                            }
                            Err(_) => 0.0,
                        }
                    } else {
                        0.0
                    };
                    
                    // Include if either score is positive
                    if keyword_score > 0 || semantic_score > 0.3 {
                        Some((doc, keyword_score, semantic_score))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse {:?}: {}", path, e);
                    None
                }
            }
        })
        .collect();

    // Compute hybrid scores and sort
    let mut scored: Vec<(Document, f32)> = candidates
        .into_iter()
        .map(|(doc, kw_score, sem_score)| {
            let normalized_kw = normalize_keyword_score(kw_score, config.keyword_max_score);
            let hybrid = sem_score * config.semantic_weight + normalized_kw * config.keyword_weight;
            (doc, hybrid)
        })
        .collect();

    // Sort by hybrid score descending
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    tracing::debug!(
        "Hybrid search found {} candidates, returning top 10",
        scored.len()
    );

    // Return top 10
    Ok(scored.into_iter().take(10).map(|(doc, _)| doc).collect())
}

/// Save the global embedding cache to disk
/// Should be called on application shutdown
pub fn save_embedding_cache() -> Result<()> {
    let cache = EMBEDDING_CACHE.lock().unwrap();
    cache.save().map_err(|e| std::io::Error::other(e.to_string()).into())
}

fn is_ignored(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    // Ignore common noise directories
    name == "node_modules"
        || name == ".git"
        || name == "target"
        || name == "dist"
        || name == "vendor"
}

fn score_file(path: &Path, query_lower: &str, tokens: &[&str]) -> i32 {
    let mut score = 0;
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    // Filename match (High weight)
    if file_name.contains(query_lower) {
        score += 100;
    }

    // Partial filename token match
    for token in tokens {
        if file_name.contains(token) {
            score += 10;
        }
    }

    // Content match
    // Read file content - ignore errors (treat as empty/unreadable)
    if let Ok(content) = std::fs::read_to_string(path) {
        let content_lower = content.to_lowercase();
        // Exact query match in content
        if content_lower.contains(query_lower) {
            score += 50;
        }

        // Token matches
        for token in tokens {
            if content_lower.contains(token) {
                score += 5;
            }
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_search_workspace() {
        let temp = assert_fs::TempDir::new().unwrap();

        temp.child("readme.md")
            .write_str("# Read Me\nWelcome to CueDeck")
            .unwrap();
        temp.child("todo.md")
            .write_str("# Todo\n- [ ] Task 1")
            .unwrap();

        // Ignored file
        temp.child("node_modules").create_dir_all().unwrap();
        temp.child("node_modules/package.json")
            .write_str("{}")
            .unwrap();

        // Create context match file
        temp.child("notes.md")
            .write_str("# Notes\nSome important legacy code")
            .unwrap();

        let root = temp.path();

        // Search for "todo"
        let results = search_workspace(root, "todo", false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].path.file_name().unwrap().to_str().unwrap(),
            "todo.md"
        );

        // Search for "legacy" (content match)
        let results = search_workspace(root, "legacy", false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].path.file_name().unwrap().to_str().unwrap(),
            "notes.md"
        );

        // Search "readme" (filename match)
        let results = search_workspace(root, "readme", false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path.file_name().unwrap(), "readme.md");
    }
}
