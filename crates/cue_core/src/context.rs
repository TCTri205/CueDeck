use crate::embedding_cache::EmbeddingCache;
use crate::embeddings::EmbeddingModel;
use crate::parse_file;
use cue_common::{Document, Result};
use lazy_static::lazy_static;
use std::cmp::Ordering;
use std::path::Path;
use std::sync::Mutex;
use walkdir::WalkDir;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

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
    /// Match if document has ANY of these tags
    pub tags: Option<Vec<String>>,
    /// Filter by priority (exact match)
    pub priority: Option<String>,
    /// Filter by assignee (exact match)
    pub assignee: Option<String>,
}

/// Paginated search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Documents in current page
    pub docs: Vec<Document>,
    /// Total count of matching documents (before pagination)
    pub total_count: usize,
    /// Opaque cursor for next page (None if no more results)
    pub next_cursor: Option<String>,
}

/// Document paired with its search relevance score
#[derive(Debug, Clone)]
pub struct DocumentWithScore {
    pub document: Document,
    /// Relevance score normalized to [0.0, 1.0] range
    pub score: f64,
}

impl SearchFilters {
    pub fn matches(&self, doc: &Document) -> bool {
        let meta = match &doc.frontmatter {
            Some(m) => m,
            None => return false,
        };

        // Filter by tags (ANY match)
        if let Some(ref filter_tags) = self.tags {
            let doc_tags = match &meta.tags {
                Some(t) => t,
                None => return false,
            };
            // Case-insensitive check
            if !filter_tags.iter().any(|ft| {
                doc_tags.iter().any(|dt| dt.eq_ignore_ascii_case(ft))
            }) {
                return false;
            }
        }

        // Filter by priority (exact match, case-insensitive)
        if let Some(ref filter_priority) = self.priority {
            if !meta.priority.eq_ignore_ascii_case(filter_priority) {
                return false;
            }
        }

        // Filter by assignee (exact match, case-insensitive)
        if let Some(ref filter_assignee) = self.assignee {
            match &meta.assignee {
                Some(a) if a.eq_ignore_ascii_case(filter_assignee) => {},
                _ => return false,
            }
        }

        true
    }
}

/// Encode pagination cursor (offset -> base64)
fn encode_cursor(offset: usize) -> String {
    BASE64.encode(offset.to_string().as_bytes())
}

/// Decode pagination cursor (base64 -> offset)
fn decode_cursor(cursor: &str) -> Result<usize> {
    let bytes = BASE64.decode(cursor)
        .map_err(|e| std::io::Error::other(format!("Invalid cursor: {}", e)))?;
    let offset_str = String::from_utf8(bytes)
        .map_err(|e| std::io::Error::other(format!("Invalid cursor encoding: {}", e)))?;
    offset_str.parse::<usize>()
        .map_err(|e| std::io::Error::other(format!("Invalid cursor value: {}", e)).into())
}

/// Main search function with mode selection
#[tracing::instrument(skip(root))]
pub fn search_workspace_with_mode(
    root: &Path,
    query: &str,
    mode: SearchMode,
    _filters: Option<SearchFilters>,
) -> Result<Vec<DocumentWithScore>> {
    tracing::info!(
        "Searching workspace: {:?} for '{}' (mode: {:?})",
        root,
        query,
        mode
    );

    match mode {
        SearchMode::Keyword => search_workspace_keyword(root, query, _filters.as_ref()),
        SearchMode::Semantic => search_workspace_semantic(root, query, 10, _filters.as_ref()),
        SearchMode::Hybrid => search_workspace_hybrid(root, query, _filters.as_ref()),
    }
}

/// Paginated search function with cursor support
#[tracing::instrument(skip(root))]
pub fn search_workspace_paginated(
    root: &Path,
    query: &str,
    mode: SearchMode,
    filters: Option<SearchFilters>,
    limit: usize,
    cursor: Option<&str>,
) -> Result<SearchResult> {
    tracing::info!(
        "Paginated search: {:?} for '{}' (mode: {:?}, limit: {}, cursor: {:?})",
        root,
        query,
        mode,
        limit,
        cursor
    );

    // Get ALL matching documents first (unpaginated) - returns DocumentWithScore
    let all_scored = match mode {
        SearchMode::Keyword => search_workspace_keyword_all(root, query, filters.as_ref()),
        SearchMode::Semantic => search_workspace_semantic_all(root, query, filters.as_ref()),
        SearchMode::Hybrid => search_workspace_hybrid_all(root, query, filters.as_ref()),
    }?;

    let total_count = all_scored.len();
    
    // Decode cursor to get offset
    let offset = match cursor {
        Some(c) => decode_cursor(c)?,
        None => 0,
    };

    // Apply pagination - extract documents from DocumentWithScore
    let end = (offset + limit).min(total_count);
    let docs: Vec<Document> = all_scored
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|scored| scored.document) // Extract document from DocumentWithScore
        .collect();

    // Generate next cursor if more results exist
    let next_cursor = if end < total_count {
        Some(encode_cursor(end))
    } else {
        None
    };

    Ok(SearchResult {
        docs,
        total_count,
        next_cursor,
    })
}

/// Backward-compatible search function (legacy API)
#[tracing::instrument(skip(root))]
pub fn search_workspace(root: &Path, query: &str, semantic: bool) -> Result<Vec<Document>> {
    let mode = if semantic {
        SearchMode::Semantic
    } else {
        SearchMode::Keyword
    };
    let results = search_workspace_with_mode(root, query, mode, None)?;
    // Extract documents from scored results for backward compatibility
    Ok(results.into_iter().map(|r| r.document).collect())
}

/// Keyword-based search (original implementation) - returns top 10
fn search_workspace_keyword(root: &Path, query: &str, filters: Option<&SearchFilters>) -> Result<Vec<DocumentWithScore>> {
    let all_results = search_workspace_keyword_all(root, query, filters)?;
    Ok(all_results.into_iter().take(10).collect())
}

/// Keyword-based search - returns ALL results (for pagination)
fn search_workspace_keyword_all(root: &Path, query: &str, filters: Option<&SearchFilters>) -> Result<Vec<DocumentWithScore>> {
    let query_lower = query.to_lowercase();
    let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();

    let mut results: Vec<(Document, i32)> = Vec::new();
    let mut max_score = 0i32;

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
                    let score = score_file(path, &query_lower, &query_tokens);

                    if score > 0 {
                        max_score = max_score.max(score);
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

    // Normalize scores to [0.0, 1.0] and filter
    let all_docs = results
        .into_iter()
        .filter(|(doc, _)| {
            filters.is_none_or(|f| f.matches(doc))
        })
        .map(|(doc, raw_score)| {
            let normalized_score = if max_score > 0 {
                (raw_score as f64) / (max_score as f64)
            } else {
                0.0
            };
            DocumentWithScore {
                document: doc,
                score: normalized_score,
            }
        })
        .collect();

    Ok(all_docs)
}

/// Semantic search using vector embeddings - returns top N
fn search_workspace_semantic(root: &Path, query: &str, limit: usize, filters: Option<&SearchFilters>) -> Result<Vec<DocumentWithScore>> {
    let all_results = search_workspace_semantic_all(root, query, filters)?;
    Ok(all_results.into_iter().take(limit).collect())
}

/// Semantic search using vector embeddings - returns ALL results (for pagination)
fn search_workspace_semantic_all(root: &Path, query: &str, filters: Option<&SearchFilters>) -> Result<Vec<DocumentWithScore>> {
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

    // Filter and return ALL results with scores
    Ok(sorted_candidates
        .into_iter()
        .filter(|(doc, _)| {
            filters.is_none_or(|f| f.matches(doc))
        })
        .map(|(doc, similarity)| DocumentWithScore {
            document: doc,
            score: similarity as f64, // Convert f32 to f64
        })
        .collect())
}

/// Normalize keyword score to [0, 1] range
fn normalize_keyword_score(raw_score: i32, max_score: i32) -> f32 {
    if max_score <= 0 {
        return 0.0;
    }
    (raw_score as f32 / max_score as f32).clamp(0.0, 1.0)
}

/// Hybrid search: combines keyword and semantic search with weighted scoring - returns top 10
fn search_workspace_hybrid(root: &Path, query: &str, filters: Option<&SearchFilters>) -> Result<Vec<DocumentWithScore>> {
    let all_results = search_workspace_hybrid_all(root, query, filters)?;
    Ok(all_results.into_iter().take(10).collect())
}

/// Hybrid search: combines keyword and semantic search - returns ALL results (for pagination)
fn search_workspace_hybrid_all(root: &Path, query: &str, filters: Option<&SearchFilters>) -> Result<Vec<DocumentWithScore>> {
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
        "Hybrid search found {} candidates",
        scored.len()
    );

    // Filter and return ALL results with scores
    Ok(scored
        .into_iter()
        .filter(|(doc, _)| {
            filters.is_none_or(|f| f.matches(doc))
        })
        .map(|(doc, hybrid_score)| DocumentWithScore {
            document: doc,
            score: hybrid_score as f64, // Convert f32 to f64
        })
        .collect())
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
