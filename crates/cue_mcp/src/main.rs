//! MCP Server Binary Entry Point
//!
//! This binary implements a JSON-RPC 2.0 server over stdin/stdout
//! following the Model Context Protocol (MCP) specification.

use std::io::{self, BufRead, Write};
use tokio::runtime::Runtime;

fn main() {
    // Initialize tracing to stderr only (stdout reserved for JSON-RPC)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("CueDeck MCP server starting...");

    // Create tokio runtime for async operations
    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    // Read requests from stdin line by line
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Error reading stdin: {}", e);
                break;
            }
        };

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        tracing::debug!("Received: {}", line);

        // Parse JSON-RPC request
        let request: cue_mcp::JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to parse request: {}", e);
                // Send parse error response
                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    }
                });
                if let Err(e) = writeln!(stdout_lock, "{}", error_response) {
                    tracing::error!("Failed to write error response: {}", e);
                    break;
                }
                if let Err(e) = stdout_lock.flush() {
                    tracing::error!("Failed to flush stdout: {}", e);
                    break;
                }
                continue;
            }
        };

        // Handle request asynchronously
        let response = rt.block_on(cue_mcp::handle_request(request));

        // Write response if not None (notifications don't get responses)
        if let Some(resp) = response {
            let response_json = match serde_json::to_string(&resp) {
                Ok(json) => json,
                Err(e) => {
                    tracing::error!("Failed to serialize response: {}", e);
                    continue;
                }
            };

            tracing::debug!("Sending: {}", response_json);

            if let Err(e) = writeln!(stdout_lock, "{}", response_json) {
                tracing::error!("Failed to write response: {}", e);
                break;
            }

            if let Err(e) = stdout_lock.flush() {
                tracing::error!("Failed to flush stdout: {}", e);
                break;
            }
        }
    }

    tracing::info!("CueDeck MCP server shutting down");
}
