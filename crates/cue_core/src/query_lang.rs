//! Unified query language parser for CueDeck
//!
//! Syntax: `field:value [field:value...] [+tag] [-tag]`
//!
//! Examples:
//! - `status:active priority:high +backend`
//! - `created:>7d updated:<2024-01-01`
//! - `assignee:@dev status:todo -archived`

use crate::{CueError, Result};
use std::collections::HashMap;

/// Parsed query structure
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedQuery {
    /// Field filters (status:active, priority:high, etc.)
    pub filters: HashMap<String, FilterValue>,
    /// Tags that must be present
    pub include_tags: Vec<String>,
    /// Tags that must NOT be present
    pub exclude_tags: Vec<String>,
}

/// Filter value types
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
    /// Exact match (status:active)
    Exact(String),
    /// Greater than (created:>7d)
    GreaterThan(String),
    /// Less than (created:<2024-01-01)
    LessThan(String),
}

/// Query language parser
pub struct QueryParser;

impl QueryParser {
    /// Parse a query string into a ParsedQuery
    ///
    /// # Examples
    ///
    /// ```
    /// use cue_core::query_lang::QueryParser;
    ///
    /// let query = QueryParser::parse("status:active +backend").unwrap();
    /// assert_eq!(query.include_tags, vec!["backend"]);
    /// ```
    pub fn parse(query_str: &str) -> Result<ParsedQuery> {
        let mut filters = HashMap::new();
        let mut include_tags = Vec::new();
        let mut exclude_tags = Vec::new();

        // Split by whitespace
        for token in query_str.split_whitespace() {
            if token.is_empty() {
                continue;
            }

            // Handle tags
            if let Some(tag) = token.strip_prefix('+') {
                if tag.is_empty() {
                    return Err(CueError::ParseError("Empty tag after '+'".to_string()));
                }
                include_tags.push(tag.to_string());
                continue;
            }

            if let Some(tag) = token.strip_prefix('-') {
                if tag.is_empty() {
                    return Err(CueError::ParseError("Empty tag after '-'".to_string()));
                }
                exclude_tags.push(tag.to_string());
                continue;
            }

            // Handle field:value pairs
            if let Some(colon_pos) = token.find(':') {
                let field = &token[..colon_pos];
                let value = &token[colon_pos + 1..];

                if field.is_empty() {
                    return Err(CueError::ParseError(format!(
                        "Empty field name in '{}'",
                        token
                    )));
                }

                if value.is_empty() {
                    return Err(CueError::ParseError(format!(
                        "Empty value for field '{}'",
                        field
                    )));
                }

                // Validate field name
                Self::validate_field(field)?;

                // Parse filter value
                let filter_value = if let Some(stripped) = value.strip_prefix('>') {
                    FilterValue::GreaterThan(stripped.to_string())
                } else if let Some(stripped) = value.strip_prefix('<') {
                    FilterValue::LessThan(stripped.to_string())
                } else {
                    FilterValue::Exact(value.to_string())
                };

                filters.insert(field.to_string(), filter_value);
            } else {
                return Err(CueError::ParseError(format!(
                    "Invalid token '{}'. Expected 'field:value', '+tag', or '-tag'",
                    token
                )));
            }
        }

        Ok(ParsedQuery {
            filters,
            include_tags,
            exclude_tags,
        })
    }

    /// Validate field name
    fn validate_field(field: &str) -> Result<()> {
        const VALID_FIELDS: &[&str] = &["status", "priority", "assignee", "created", "updated"];

        if !VALID_FIELDS.contains(&field) {
            return Err(CueError::ParseError(format!(
                "Unknown field '{}'. Valid fields: {}",
                field,
                VALID_FIELDS.join(", ")
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_filter() {
        let query = QueryParser::parse("status:active").unwrap();
        assert_eq!(
            query.filters.get("status"),
            Some(&FilterValue::Exact("active".to_string()))
        );
        assert!(query.include_tags.is_empty());
        assert!(query.exclude_tags.is_empty());
    }

    #[test]
    fn test_parse_tags() {
        let query = QueryParser::parse("+backend -archived").unwrap();
        assert_eq!(query.include_tags, vec!["backend"]);
        assert_eq!(query.exclude_tags, vec!["archived"]);
        assert!(query.filters.is_empty());
    }

    #[test]
    fn test_parse_complex_query() {
        let query = QueryParser::parse("status:active priority:high +backend -archived").unwrap();

        assert_eq!(
            query.filters.get("status"),
            Some(&FilterValue::Exact("active".to_string()))
        );
        assert_eq!(
            query.filters.get("priority"),
            Some(&FilterValue::Exact("high".to_string()))
        );
        assert_eq!(query.include_tags, vec!["backend"]);
        assert_eq!(query.exclude_tags, vec!["archived"]);
    }

    #[test]
    fn test_parse_relative_date() {
        let query = QueryParser::parse("created:>7d updated:<30d").unwrap();

        assert_eq!(
            query.filters.get("created"),
            Some(&FilterValue::GreaterThan("7d".to_string()))
        );
        assert_eq!(
            query.filters.get("updated"),
            Some(&FilterValue::LessThan("30d".to_string()))
        );
    }

    #[test]
    fn test_parse_assignee() {
        let query = QueryParser::parse("assignee:@dev status:todo").unwrap();

        assert_eq!(
            query.filters.get("assignee"),
            Some(&FilterValue::Exact("@dev".to_string()))
        );
    }

    #[test]
    fn test_invalid_field() {
        let result = QueryParser::parse("invalid_field:value");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown field 'invalid_field'"));
    }

    #[test]
    fn test_empty_tag() {
        let result = QueryParser::parse("+");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty tag"));
    }

    #[test]
    fn test_invalid_token() {
        let result = QueryParser::parse("invalidtoken");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid token"));
    }

    #[test]
    fn test_empty_query() {
        let query = QueryParser::parse("").unwrap();
        assert!(query.filters.is_empty());
        assert!(query.include_tags.is_empty());
        assert!(query.exclude_tags.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let query = QueryParser::parse("   ").unwrap();
        assert!(query.filters.is_empty());
    }
}
