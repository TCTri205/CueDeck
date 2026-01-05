//! Query command implementation using unified query language

use anyhow::Result;
use serde_json;

/// Execute query using unified query language
pub async fn cmd_query(
    query: Option<String>,
    batch_file: Option<String>,
    json: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;

    if let Some(file_path) = batch_file {
        // Batch mode: read queries from JSON file
        let file_content = std::fs::read_to_string(&file_path)?;
        let batch: cue_core::BatchQuery = serde_json::from_str(&file_content)?;

        let response = cue_core::batch_query(&cwd, &batch)?;

        // Always JSON output for batch mode
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else if let Some(query_str) = query {
        // Single query mode
        let parsed = cue_core::QueryParser::parse(&query_str)?;

        // Build filters from parsed query
        let mut filters = cue_core::TaskFilters::default();

        for (field, value) in &parsed.filters {
            match field.as_str() {
                "status" => {
                    if let cue_core::FilterValue::Exact(val) = value {
                        filters.status = Some(val.clone());
                    }
                }
                "priority" => {
                    if let cue_core::FilterValue::Exact(val) = value {
                        filters.priority = Some(val.clone());
                    }
                }
                "assignee" => {
                    if let cue_core::FilterValue::Exact(val) = value {
                        filters.assignee = Some(val.clone());
                    }
                }
                "created" => {
                    filters.created = parse_filter_value_to_date(value)?;
                }
                "updated" => {
                    filters.updated = parse_filter_value_to_date(value)?;
                }
                _ => {}
            }
        }

        // Add tag filters
        filters.tags = if !parsed.include_tags.is_empty() {
            Some(parsed.include_tags.clone())
        } else {
            None
        };

        // Execute query
        let mut tasks = cue_core::tasks::list_tasks_filtered(&cwd, &filters)?;

        // Filter out excluded tags
        if !parsed.exclude_tags.is_empty() {
            tasks.retain(|doc| {
                if let Some(ref meta) = doc.frontmatter {
                    if let Some(ref tags) = meta.tags {
                        return !parsed.exclude_tags.iter().any(|exclude| tags.contains(exclude));
                    }
                }
                true
            });
        }

        if json {
            // JSON output
            let json_tasks: Vec<_> = tasks
                .iter()
                .map(|doc| {
                    let id = doc.path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");

                    let meta = doc.frontmatter.as_ref();

                    serde_json::json!({
                        "id": id,
                        "title": meta.map(|m| m.title.as_str()).unwrap_or("Untitled"),
                        "status": meta.map(|m| m.status.as_str()).unwrap_or("unknown"),
                        "priority": meta.map(|m| m.priority.as_str()).unwrap_or("medium"),
                        "tags": meta.and_then(|m| m.tags.as_ref()),
                    })
                })
                .collect();

            println!("{}", serde_json::to_string_pretty(&json_tasks)?);
        } else {
            // Human-readable output
            eprintln!("Found {} tasks matching query: '{}'", tasks.len(), query_str);
            for (i, doc) in tasks.iter().enumerate() {
                let meta = doc.frontmatter.as_ref();
                let title = meta.map(|m| m.title.as_str()).unwrap_or("Untitled");
                let status = meta.map(|m| m.status.as_str()).unwrap_or("unknown");
                eprintln!("  {}. [{}] {}", i + 1, status, title);
            }
        }
    } else {
        anyhow::bail!("Must provide either query string or --batch file");
    }

    Ok(())
}

/// Helper to convert FilterValue to DateFilter
fn parse_filter_value_to_date(
    value: &cue_core::FilterValue,
) -> Result<Option<cue_core::DateFilter>> {
    use cue_core::task_filters::{DateFilter, DateOperator, DateValue};

    match value {
        cue_core::FilterValue::GreaterThan(rel) => {
            if let Some(days_str) = rel.strip_suffix('d') {
                let days: i64 = days_str.parse()?;
                Ok(Some(DateFilter {
                    operator: DateOperator::After,
                    value: DateValue::Relative(chrono::Duration::days(days)),
                }))
            } else {
                Ok(None)
            }
        }
        cue_core::FilterValue::LessThan(rel) => {
            if let Some(days_str) = rel.strip_suffix('d') {
                let days: i64 = days_str.parse()?;
                Ok(Some(DateFilter {
                    operator: DateOperator::Before,
                    value: DateValue::Relative(chrono::Duration::days(days)),
                }))
            } else {
                Ok(None)
            }
        }
        cue_core::FilterValue::Exact(date_str) => {
            // Try parsing as absolute date
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(Some(DateFilter {
                    operator: DateOperator::Equals,
                    value: DateValue::Absolute(date),
                }))
            } else {
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_requires_input() {
        let result = cmd_query(None, None, false).await;
        assert!(result.is_err());
    }
}
