//! Telemetry and logging initialization
//!
//! Provides structured logging with `tracing` and `tracing-subscriber`.
//! Per LOGGING_AND_TELEMETRY.md: stdout is reserved for JSON-RPC, all logs go to stderr.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing subscriber with appropriate configuration
///
/// # Arguments
/// * `verbose` - If true, sets log level to DEBUG, otherwise INFO
/// * `json_format` - If true, outputs logs in JSON format for machine parsing
///
/// # Example
/// ```
/// cue_common::telemetry::init_tracing(false, false);
/// tracing::info!("Application started");
/// ```
pub fn init_tracing(verbose: bool, json_format: bool) {
    let filter_level = if verbose { 
        "debug,hyper=info,tokio=info" 
    } else { 
        "info" 
    };
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter_level));
    
    if json_format {
        // JSON format for machine-parseable logs
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(std::io::stderr) // CRITICAL: Never write to stdout
            )
            .with(env_filter)
            .init();
    } else {
        // Human-readable format for development
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stderr) // CRITICAL: Never write to stdout
                    .with_target(false)
                    .compact()
            )
            .with(env_filter)
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    #[test]
    fn test_init_tracing() {
        // We can only initialize tracing ONCE per process (shared across all tests in this binary)
        INIT.call_once(|| {
            init_tracing(false, false);
        });
        // If we get here without panic, it works.
        // Subsequent calls would panic if we didn't guard them, or if we called init_tracing again directly.
    }
}
