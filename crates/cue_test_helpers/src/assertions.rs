//! Domain-specific assertions for CueDeck tests
//!
//! Provides custom predicates and assertion helpers for common
//! test patterns in CueDeck.

use predicates::prelude::*;
use predicates::str::contains;

/// Assert that stderr does NOT contain any of the given strings
///
/// Useful for verifying that certain log messages or errors don't appear.
///
/// # Arguments
///
/// * `values` - Slice of strings that should NOT appear in stderr
///
/// # Example
///
/// ```rust
/// use cue_test_helpers::assertions::stderr_not_contains;
/// use predicates::prelude::*;
///
/// let output = std::process::Command::new("cue")
///     .arg("list")
///     .output()
///     .unwrap();
///
/// let stderr = String::from_utf8_lossy(&output.stderr);
/// assert!(stderr_not_contains(&["ERROR", "WARN"]).eval(&stderr));
/// ```
pub fn stderr_not_contains(values: &[&str]) -> impl Predicate<str> {
    let owned_values: Vec<String> = values.iter().map(|&s| s.to_string()).collect();
    predicate::function(move |s: &str| {
        !owned_values.iter().any(|v| s.contains(v.as_str()))
    })
}

/// Assert that a string is valid JSON-RPC response
///
/// Checks for basic JSON-RPC structure (jsonrpc field, id, result or error).
///
/// # Example
/// ```rust
/// use cue_test_helpers::assertions::valid_jsonrpc_response;
/// use predicates::prelude::*;
///
/// let response = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
/// assert!(valid_jsonrpc_response().eval(response));
/// ```
pub fn valid_jsonrpc_response() -> impl Predicate<str> {
    contains("\"jsonrpc\"")
        .and(contains("\"id\""))
        .and(contains("\"result\"").or(contains("\"error\"")))
}

/// Assert that a string contains a valid task ID format
///
/// CueDeck task IDs are 6-character alphanumeric strings.
///
/// # Example
/// ```rust
/// use cue_test_helpers::assertions::contains_task_id;
/// use predicates::prelude::*;
///
/// let output = "✓ Created task: abc123 at ...";
/// assert!(contains_task_id().eval(output));
/// ```
pub fn contains_task_id() -> impl Predicate<str> {
    predicate::function(|s: &str| {
        s.chars()
            .collect::<Vec<_>>()
            .windows(6)
            .any(|window| window.iter().all(|c| c.is_alphanumeric()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stderr_not_contains() {
        let stderr = "Some output without errors";
        assert!(stderr_not_contains(&["ERROR", "WARN"]).eval(stderr));

        let stderr_with_error = "ERROR: something went wrong";
        assert!(!stderr_not_contains(&["ERROR"]).eval(stderr_with_error));
    }

    #[test]
    fn test_valid_jsonrpc_response() {
        let valid = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
        assert!(valid_jsonrpc_response().eval(valid));

        let valid_error = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600}}"#;
        assert!(valid_jsonrpc_response().eval(valid_error));
        
        let invalid = r#"{"data":"test"}"#;
        assert!(!valid_jsonrpc_response().eval(invalid));
    }

    #[test]
    fn test_contains_task_id() {
        assert!(contains_task_id().eval("Created task abc123"));
        assert!(contains_task_id().eval("abc123"));
        assert!(!contains_task_id().eval("abc12")); // Too short
        assert!(!contains_task_id().eval("no task id here"));
    }
}
