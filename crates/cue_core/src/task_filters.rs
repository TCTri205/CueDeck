//! Task filtering types and logic for advanced search

use cue_common::Result;

/// Advanced task filters for list_tasks_filtered()
#[derive(Debug, Default, Clone)]
pub struct TaskFilters {
    /// Status filter (active, done, archived, etc.)
    pub status: Option<String>,
    
    /// Assignee filter
    pub assignee: Option<String>,
    
    /// Tag filters (comma-separated = OR logic)
    pub tags: Option<Vec<String>>,
    
    /// Priority filter
    pub priority: Option<String>,
    
    /// Created date filter
    pub created: Option<DateFilter>,
    
    /// Updated date filter
    pub updated: Option<DateFilter>,
}

/// Date filter with operator and value
#[derive(Debug, Clone)]
pub struct DateFilter {
    pub operator: DateOperator,
    pub value: DateValue,
}

/// Date comparison operators
#[derive(Debug, Clone, PartialEq)]
pub enum DateOperator {
    /// Equals (created:2024-01-01)
    Equals,
    /// Before (created<2024)
    Before,
    /// After (created>2024)
    After,
    /// Within relative time (updated>2w)
    Within,
}

/// Date value (absolute or relative)
#[derive(Debug, Clone)]
pub enum DateValue {
    /// Absolute date (2024-01-01)
    Absolute(chrono::NaiveDate),
    /// Relative duration (2w, 7d, 1m)
    Relative(chrono::Duration),
}

/// Parse tag filter from comma-separated string
/// Example: "auth,api,backend" -> ["auth", "api", "backend"]
pub fn parse_tag_filter(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse date filter from string
/// Examples: "2024-01-01", ">2w", "<7d", "2024"
pub fn parse_date_filter(input: &str) -> Result<DateFilter> {
    if input.starts_with('>') {
        Ok(DateFilter {
            operator: DateOperator::After,
            value: parse_date_value(&input[1..])?,
        })
    } else if input.starts_with('<') {
        Ok(DateFilter {
            operator: DateOperator::Before,
            value: parse_date_value(&input[1..])?,
        })
    } else {
        Ok(DateFilter {
            operator: DateOperator::Equals,
            value: parse_date_value(input)?,
        })
    }
}

/// Parse date value (absolute or relative)
fn parse_date_value(input: &str) -> Result<DateValue> {
    // Try relative duration first (2w, 7d, 1m, 3y)
    if let Some(dur) = parse_relative_duration(input) {
        return Ok(DateValue::Relative(dur));
    }

    // Try absolute dates: YYYY-MM-DD, YYYY-MM, YYYY
    if let Ok(date) = chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return Ok(DateValue::Absolute(date));
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{}-01", input), "%Y-%m-%d") {
        return Ok(DateValue::Absolute(date)); // YYYY-MM
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{}-01-01", input), "%Y-%m-%d") {
        return Ok(DateValue::Absolute(date)); // YYYY
    }

    Err(cue_common::CueError::ParseError(format!(
        "Invalid date format: '{}'. Expected: YYYY-MM-DD, YYYY-MM, YYYY, or relative (7d, 2w, 3m, 1y)",
        input
    )))
}

/// Parse relative duration (7d, 2w, 3m, 1y)
fn parse_relative_duration(input: &str) -> Option<chrono::Duration> {
    let re = regex::Regex::new(r"^(\d+)([dwmy])$").ok()?;
    let caps = re.captures(input)?;
    let num: i64 = caps.get(1)?.as_str().parse().ok()?;
    let unit = caps.get(2)?.as_str();

    match unit {
        "d" => Some(chrono::Duration::days(num)),
        "w" => Some(chrono::Duration::weeks(num)),
        "m" => Some(chrono::Duration::days(num * 30)), // Approximate
        "y" => Some(chrono::Duration::days(num * 365)), // Approximate
        _ => None,
    }
}

/// Check if task tags match filter (ANY match = OR logic)
pub fn matches_tag_filter(task_tags: &Option<Vec<String>>, filter_tags: &[String]) -> bool {
    match task_tags {
        None => false,
        Some(tags) => {
            // ANY match (OR logic)
            filter_tags.iter().any(|ft| {
                tags.iter().any(|t| t.eq_ignore_ascii_case(ft))
            })
        }
    }
}

/// Check if task date matches filter
pub fn matches_date_filter(
    date_str: &Option<String>,
    filter: &DateFilter,
) -> Result<bool> {
    let task_date = match date_str {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.date_naive())
            .ok(),
        None => return Ok(false),
    };

    let Some(task_date) = task_date else {
        return Ok(false);
    };

    let now = chrono::Utc::now().date_naive();

    match (&filter.operator, &filter.value) {
        (DateOperator::Equals, DateValue::Absolute(date)) => Ok(task_date == *date),
        (DateOperator::Before, DateValue::Absolute(date)) => Ok(task_date < *date),
        (DateOperator::After, DateValue::Absolute(date)) => Ok(task_date > *date),
        (DateOperator::Within, DateValue::Relative(dur)) => {
            let cutoff = now - *dur;
            Ok(task_date > cutoff)
        }
        _ => Ok(false),
    }
}

/// Check if file modification time matches filter (fallback for 'updated')
pub fn matches_date_filter_mtime(
    mtime: &std::time::SystemTime,
    filter: &DateFilter,
) -> Result<bool> {
    use std::time::UNIX_EPOCH;
    
    let duration_since_epoch = mtime.duration_since(UNIX_EPOCH)
        .map_err(|e| cue_common::CueError::ParseError(format!("Invalid mtime: {}", e)))?;
    
    let seconds = duration_since_epoch.as_secs() as i64;
    let datetime = chrono::DateTime::from_timestamp(seconds, 0)
        .ok_or_else(|| cue_common::CueError::ParseError("Invalid timestamp".to_string()))?;
    let file_date = datetime.date_naive();

    let now = chrono::Utc::now().date_naive();

    match (&filter.operator, &filter.value) {
        (DateOperator::Equals, DateValue::Absolute(date)) => Ok(file_date == *date),
        (DateOperator::Before, DateValue::Absolute(date)) => Ok(file_date < *date),
        (DateOperator::After, DateValue::Absolute(date)) => Ok(file_date > *date),
        (DateOperator::Within, DateValue::Relative(dur)) => {
            let cutoff = now - *dur;
            Ok(file_date > cutoff)
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_filter() {
        let tags = parse_tag_filter("auth,api,backend");
        assert_eq!(tags, vec!["auth", "api", "backend"]);

        let tags = parse_tag_filter(" frontend , ui ");
        assert_eq!(tags, vec!["frontend", "ui"]);

        let tags = parse_tag_filter("single");
        assert_eq!(tags, vec!["single"]);
    }

    #[test]
    fn test_parse_relative_duration() {
        assert!(parse_relative_duration("7d").is_some());
        assert!(parse_relative_duration("2w").is_some());
        assert!(parse_relative_duration("3m").is_some());
        assert!(parse_relative_duration("1y").is_some());
        assert!(parse_relative_duration("invalid").is_none());
    }

    #[test]
    fn test_parse_date_filter() {
        // Relative
        let filter = parse_date_filter(">2w").unwrap();
        assert_eq!(filter.operator, DateOperator::After);

        let filter = parse_date_filter("<7d").unwrap();
        assert_eq!(filter.operator, DateOperator::Before);

        // Absolute
        let filter = parse_date_filter("2024-01-01").unwrap();
        assert_eq!(filter.operator, DateOperator::Equals);

        let filter = parse_date_filter("2024").unwrap();
        assert_eq!(filter.operator, DateOperator::Equals);
    }

    #[test]
    fn test_matches_tag_filter() {
        let task_tags = Some(vec!["auth".to_string(), "backend".to_string()]);
        let filter_tags = vec!["auth".to_string()];
        assert!(matches_tag_filter(&task_tags, &filter_tags));

        let filter_tags = vec!["frontend".to_string()];
        assert!(!matches_tag_filter(&task_tags, &filter_tags));

        let task_tags = None;
        assert!(!matches_tag_filter(&task_tags, &filter_tags));
    }
}
