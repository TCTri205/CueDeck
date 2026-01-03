//! Test logging configuration utilities
//!
//! Provides functions to configure tracing/logging for tests
//! to prevent output pollution and enable debugging when needed.

use tracing_subscriber::{EnvFilter, FmtSubscriber};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize tracing for tests with custom log level
///
/// This function can only be called once per test process due to global
/// subscriber limitations. Subsequent calls are ignored.
///
/// # Arguments
///
/// * `level` - Log level filter (e.g., "debug", "info", "warn", "error")
///
/// # Example
///
/// ```rust
/// use cue_test_helpers::logging::init_test_logging;
///
/// fn my_test() {
///     init_test_logging("debug");
///     // Test code with debug logging enabled
/// }
/// ```
pub fn init_test_logging(level: &str) {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level));
        
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(filter)
            .with_test_writer()
            .finish();
        
        let _ = tracing::subscriber::set_global_default(subscriber);
    });
}

/// Suppress all logs for clean test output
///
/// Equivalent to `init_test_logging("error")` but more explicit.
///
/// # Example
///
/// ```rust
/// use cue_test_helpers::logging::suppress_logs;
///
/// fn my_test() {
///     suppress_logs();
///     // Test code with minimal logging
/// }
/// ```
pub fn suppress_logs() {
    init_test_logging("error");
}
