//! Batch query API for efficient multi-query execution
//!
//! This module provides batch query functionality to execute multiple
//! queries against a workspace in a single operation, sharing the cost
//! of workspace scanning.

use crate::query_lang::{FilterValue, ParsedQuery, QueryParser};
use crate::task_filters::{matches_date_filter, matches_tag_filter, DateFilter};
use crate::{parse_file, CueError, Document, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

/// Batch query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchQuery {
    pub queries: Vec<Query>,
}

/// Individual query within a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Client-provided ID for response matching
    pub id: String,
    /// Query string in unified query language
    pub query_string: String,
    /// Optional result limit per query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Batch query response
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchResponse {
    pub results: Vec<QueryResult>,
    pub execution_time_ms: u64,
    pub total_documents_scanned: usize,
}

/// Individual query result
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    /// Matches Query.id
    pub id: String,
    /// Matching documents
    pub documents: Vec<Document>,
    /// Total count (before limit applied)
    pub count: usize,
    /// Error if query failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Execute a batch of queries against the workspace
///
/// # Performance
///
/// This function is optimized for batch execution:
/// - Single workspace scan (shared cost)
/// - Parallel query evaluation (Rayon)
/// - Result deduplication
///
/// Target: <100ms for 50 queries
pub fn batch_query(workspace_root: &Path, batch: &BatchQuery) -> Result<BatchResponse> {
    let start = Instant::now();

    // 1. Single workspace scan (shared cost - biggest optimization)
    let all_docs = scan_workspace_once(workspace_root)?;
    let total_docs = all_docs.len();

    // 2. Parse all queries upfront
    let parsed_queries: Vec<_> = batch
        .queries
        .iter()
        .map(|q| {
            (
                q.id.clone(),
                q.limit,
                QueryParser::parse(&q.query_string),
            )
        })
        .collect();

    // 3. Execute queries in parallel using Rayon
    let results: Vec<QueryResult> = parsed_queries
        .par_iter()
        .map(|(id, limit, parsed_result)| match parsed_result {
            Ok(parsed) => {
                let mut matches = filter_documents(&all_docs, parsed);
                let total_count = matches.len();

                // Apply limit if specified
                if let Some(lim) = limit {
                    matches.truncate(*lim);
                }

                QueryResult {
                    id: id.clone(),
                    documents: matches,
                    count: total_count,
                    error: None,
                }
            }
            Err(e) => QueryResult {
                id: id.clone(),
                documents: vec![],
                count: 0,
                error: Some(e.to_string()),
            },
        })
        .collect();

    Ok(BatchResponse {
        results,
        execution_time_ms: start.elapsed().as_millis() as u64,
        total_documents_scanned: total_docs,
    })
}

/// Scan workspace once and return all documents
fn scan_workspace_once(workspace_root: &Path) -> Result<Vec<Document>> {
    let cards_dir = workspace_root.join(".cuedeck/cards");

    if !cards_dir.exists() {
        return Ok(vec![]);
    }

    let mut documents = Vec::new();

    for entry in walkdir::WalkDir::new(&cards_dir)
        .max_depth(1)
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

    Ok(documents)
}

/// Filter documents based on parsed query
fn filter_documents(documents: &[Document], query: &ParsedQuery) -> Vec<Document> {
    documents
        .iter()
        .filter(|doc| matches_query(doc, query))
        .cloned()
        .collect()
}

/// Check if a document matches the query
fn matches_query(doc: &Document, query: &ParsedQuery) -> bool {
    let Some(meta) = &doc.frontmatter else {
        return false;
    };

    // Check all field filters
    for (field, filter_value) in &query.filters {
        let matches = match field.as_str() {
            "status" => matches_field(&meta.status, filter_value),
            "priority" => matches_field(&meta.priority, filter_value),
            "assignee" => meta
                .assignee
                .as_ref()
                .map(|a| matches_field(a, filter_value))
                .unwrap_or(false),
            "created" => matches_date_field(&meta.created, filter_value),
            "updated" => matches_date_field(&meta.updated, filter_value),
            _ => false, // Unknown field (should have been caught by parser)
        };

        if !matches {
            return false;
        }
    }

    // Check include tags (must have ALL)
    // Tag filters
    if !query.include_tags.is_empty()
        && !matches_tag_filter(&meta.tags, &query.include_tags)
    {
        return false;
    }

    // Check exclude tags (must NOT have ANY)
    for exclude_tag in &query.exclude_tags {
        if let Some(tags) = &meta.tags {
            if tags.contains(exclude_tag) {
                return false;
            }
        }
    }

    true
}

/// Check if a field value matches the filter
fn matches_field(value: &str, filter: &FilterValue) -> bool {
    match filter {
        FilterValue::Exact(expected) => value.eq_ignore_ascii_case(expected),
        // For non-date fields, > and < don't make sense, always return false
        FilterValue::GreaterThan(_) | FilterValue::LessThan(_) => false,
    }
}

/// Check if a date field matches the filter
fn matches_date_field(value: &Option<String>, filter: &FilterValue) -> bool {
    match filter {
        FilterValue::Exact(expected) => value
            .as_ref()
            .map(|v| v.starts_with(expected))
            .unwrap_or(false),
        FilterValue::GreaterThan(rel) | FilterValue::LessThan(rel) => {
            // Convert to DateFilter and use existing logic
            let op = if matches!(filter, FilterValue::GreaterThan(_)) {
                crate::task_filters::DateOperator::After
            } else {
                crate::task_filters::DateOperator::Before
            };

            // Parse relative date (e.g., "7d", "30d")
            if let Ok(date_filter) = parse_relative_date(rel, op) {
                matches_date_filter(value, &date_filter).unwrap_or(false)
            } else {
                false
            }
        }
    }
}

/// Parse relative date string (e.g., "7d", "30d") into DateFilter
fn parse_relative_date(
    rel: &str,
    op: crate::task_filters::DateOperator,
) -> Result<DateFilter> {
    use crate::task_filters::{DateFilter, DateValue};

    if let Some(days_str) = rel.strip_suffix('d') {
        let days: i64 = days_str
            .parse()
            .map_err(|_| CueError::ParseError(format!("Invalid relative date: {}", rel)))?;

        Ok(DateFilter {
            operator: op,
            value: DateValue::Relative(chrono::Duration::days(days)),
        })
    } else {
        Err(CueError::ParseError(format!(
            "Unsupported date format: {}. Use format like '7d' or '30d'",
            rel
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    fn create_test_workspace() -> assert_fs::TempDir {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();

        // Create test tasks
        cards_dir
            .child("task1.md")
            .write_str(
                "---\n\
                title: Backend Task\n\
                status: active\n\
                priority: high\n\
                tags: [backend, api]\n\
                ---\n\
                # Backend Task",
            )
            .unwrap();

        cards_dir
            .child("task2.md")
            .write_str(
                "---\n\
                title: Frontend Task\n\
                status: todo\n\
                priority: medium\n\
                tags: [frontend]\n\
                ---\n\
                # Frontend Task",
            )
            .unwrap();

        cards_dir
            .child("task3.md")
            .write_str(
                "---\n\
                title: Done Task\n\
                status: done\n\
                priority: low\n\
                tags: [backend]\n\
                ---\n\
                # Done Task",
            )
            .unwrap();

        temp
    }

    #[test]
    fn test_batch_query_simple() {
        let workspace = create_test_workspace();

        let batch = BatchQuery {
            queries: vec![
                Query {
                    id: "q1".to_string(),
                    query_string: "status:active".to_string(),
                    limit: None,
                },
                Query {
                    id: "q2".to_string(),
                    query_string: "status:todo".to_string(),
                    limit: None,
                },
            ],
        };

        let response = batch_query(workspace.path(), &batch).unwrap();

        assert_eq!(response.results.len(), 2);
        assert_eq!(response.total_documents_scanned, 3);

        // Check q1 result (status:active)
        let q1_result = response.results.iter().find(|r| r.id == "q1").unwrap();
        assert_eq!(q1_result.count, 1);
        assert!(q1_result.error.is_none());

        // Check q2 result (status:todo)
        let q2_result = response.results.iter().find(|r| r.id == "q2").unwrap();
        assert_eq!(q2_result.count, 1);
    }

    #[test]
    fn test_batch_query_with_tags() {
        let workspace = create_test_workspace();

        let batch = BatchQuery {
            queries: vec![Query {
                id: "q1".to_string(),
                query_string: "+backend".to_string(),
                limit: None,
            }],
        };

        let response = batch_query(workspace.path(), &batch).unwrap();
        let result = &response.results[0];

        assert_eq!(result.count, 2); // task1 and task3 have backend tag
    }

    #[test]
    fn test_batch_query_with_limit() {
        let workspace = create_test_workspace();

        let batch = BatchQuery {
            queries: vec![Query {
                id: "q1".to_string(),
                query_string: "+backend".to_string(),
                limit: Some(1),
            }],
        };

        let response = batch_query(workspace.path(), &batch).unwrap();
        let result = &response.results[0];

        assert_eq!(result.count, 2); // Total matches
        assert_eq!(result.documents.len(), 1); // Limited to 1
    }

    #[test]
    fn test_batch_query_invalid_syntax() {
        let workspace = create_test_workspace();

        let batch = BatchQuery {
            queries: vec![Query {
                id: "q1".to_string(),
                query_string: "invalid_syntax".to_string(),
                limit: None,
            }],
        };

        let response = batch_query(workspace.path(), &batch).unwrap();
        let result = &response.results[0];

        assert!(result.error.is_some());
        assert_eq!(result.count, 0);
    }

    #[test]
    fn test_batch_query_performance() {
        let workspace = create_test_workspace();

        // Create 50 queries
        let queries: Vec<Query> = (0..50)
            .map(|i| Query {
                id: format!("q{}", i),
                query_string: "status:active".to_string(),
                limit: Some(10),
            })
            .collect();

        let batch = BatchQuery { queries };

        let start = Instant::now();
        let response = batch_query(workspace.path(), &batch).unwrap();
        let elapsed = start.elapsed();

        println!("Batch query 50 queries took: {:?}", elapsed);
        assert_eq!(response.results.len(), 50);
        // Performance target: <100ms for 50 queries
        // This is a small workspace, so should be much faster
        assert!(elapsed.as_millis() < 100);
    }
}
