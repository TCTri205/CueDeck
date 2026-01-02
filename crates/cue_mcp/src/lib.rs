//! MCP (Model Context Protocol) server implementation
//!
//! This crate provides the JSON-RPC server for AI agent integration.
//!
//! CRITICAL: stdout is reserved EXCLUSIVELY for JSON-RPC responses.
//! All logs (Info/Warn/Error) MUST go to stderr to avoid protocol corruption.

use cue_common::{CueError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// JSON-RPC request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// JSON-RPC response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error object
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Convert CueError to JSON-RPC error
    pub fn from_cue_error(err: &CueError) -> Self {
        let code = match err {
            CueError::FileNotFound { .. } => 1001,
            CueError::CycleDetected => 1002,
            CueError::TokenLimit { .. } => 1003,
            CueError::StaleCache => 1006,
            CueError::Locked { .. } => 1007,
            CueError::RateLimit { .. } => 429,
            CueError::ValidationError(_) => -32602, // Invalid params
            CueError::ParseError(_) => -32700,      // Parse error
            _ => -32603,                            // Internal error
        };

        let message = err.to_string();

        // Add structured data for specific errors
        let data = match err {
            CueError::RateLimit {
                current,
                limit,
                window,
            } => Some(serde_json::json!({
                "retry_after_seconds": window - 1,
                "limit": limit,
                "window_seconds": window,
                "current_count": current
            })),
            CueError::FileNotFound { path } => Some(serde_json::json!({
                "path": path
            })),
            _ => None,
        };

        Self {
            code,
            message,
            data,
        }
    }
}

/// Rate limiter for MCP methods
struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn check_rate_limit(&self, method: &str) -> Result<()> {
        let (limit, window) = match method {
            "read_context" => (10, 60),
            "read_doc" => (30, 60),
            "list_tasks" => (20, 60),
            "update_task" => (10, 60),
            _ => return Ok(()),
        };

        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        let method_requests = requests.entry(method.to_string()).or_default();

        // Remove requests older than window
        method_requests.retain(|&time| now.duration_since(time).as_secs() < window);

        if method_requests.len() >= limit {
            let _retry_after = window - now.duration_since(method_requests[0]).as_secs();
            return Err(CueError::RateLimit {
                current: method_requests.len(),
                limit,
                window,
            });
        }

        method_requests.push(now);
        Ok(())
    }
}

/// Global rate limiter instance
static RATE_LIMITER: once_cell::sync::Lazy<RateLimiter> =
    once_cell::sync::Lazy::new(RateLimiter::new);

/// Handle a single JSON-RPC request
pub async fn handle_request(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    // Log to stderr only
    tracing::info!(target: "mcp", method = %request.method, "Handling MCP request");

    // Handle notifications (no id)
    if request.id.is_none() {
        // Special case: ignore known notifications
        if request.method == "notifications/initialized" {
            return None;
        }
    }

    // Check rate limit
    if let Err(e) = RATE_LIMITER.check_rate_limit(&request.method) {
        return Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError::from_cue_error(&e)),
        });
    }

    // Dispatch to handler
    let result = match request.method.as_str() {
        "initialize" => handle_initialize(request.params).await,
        "tools/list" => handle_tools_list().await,
        "tools/call" => handle_tools_call(request.params).await,
        "ping" => handle_ping().await,
        "read_context" => handle_read_context(request.params).await,
        "read_doc" => handle_read_doc(request.params).await,
        "list_tasks" => handle_list_tasks(request.params).await,
        "create_task" => handle_create_task(request.params).await,
        "update_task" => handle_update_task(request.params).await,
        // Ignore notifications we don't care about
        _ if request.id.is_none() => return None,
        _ => Err(CueError::ValidationError(format!(
            "Unknown method: {}",
            request.method
        ))),
    };

    match result {
        Ok(value) => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        }),
        Err(e) => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError::from_cue_error(&e)),
        }),
    }
}

/// Initialize handler - MCP protocol handshake
async fn handle_initialize(_params: Option<Value>) -> Result<Value> {
    Ok(serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "cuedeck",
            "version": env!("CARGO_PKG_VERSION")
        }
    }))
}

/// Tools call handler - dispatch to specific tool implementation
async fn handle_tools_call(params: Option<Value>) -> Result<Value> {
    let params = params.ok_or_else(|| {
        CueError::ValidationError("Missing params for tools/call".to_string())
    })?;

    let name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
        CueError::ValidationError("Missing 'name' field in tools/call params".to_string())
    })?;

    let args = params.get("arguments").cloned();

    let result = match name {
        "read_context" => handle_read_context(args).await?,
        "read_doc" => handle_read_doc(args).await?,
        "list_tasks" => handle_list_tasks(args).await?,
        "create_task" => handle_create_task(args).await?,
        "update_task" => handle_update_task(args).await?,
        _ => {
            return Err(CueError::ValidationError(format!(
                "Unknown tool: {}",
                name
            )))
        }
    };

    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&result).unwrap_or_default()
            }
        ]
    }))
}

/// Tools list handler - return available MCP tools
async fn handle_tools_list() -> Result<Value> {
    Ok(serde_json::json!({
        "tools": [
            {
                "name": "read_context",
                "description": "Smart fuzzy search across project context headers and filenames",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Keywords to match (e.g. 'auth flow')"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max number of results (default: 5)",
                            "default": 5
                        },
                        "semantic": {
                            "type": "boolean",
                            "description": "Use semantic search with embeddings",
                            "default": false
                        },
                        "filters": {
                            "type": "object",
                            "description": "Optional filters for results",
                            "properties": {
                                "tags": {
                                    "type": "array",
                                    "items": { "type": "string" },
                                    "description": "Filter by tags (ANY match)"
                                },
                                "priority": {
                                    "type": "string",
                                    "description": "Filter by priority (exact match)"
                                },
                                "assignee": {
                                    "type": "string",
                                    "description": "Filter by assignee (exact match)"
                                }
                            }
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "read_doc",
                "description": "Read a specific document or section of it",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path to markdown file"
                        },
                        "anchor": {
                            "type": "string",
                            "description": "Optional header name to slice from"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "list_tasks",
                "description": "List task cards filtered by status",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["todo", "active", "done", "archived"],
                            "description": "Filter by task status"
                        },
                        "assignee": {
                            "type": "string",
                            "description": "Filter by assignee name"
                        }
                    }
                }
            },
            {
                "name": "update_task",
                "description": "Update task card metadata",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "6-character task ID"
                        },
                        "updates": {
                            "type": "object",
                            "description": "Fields to update (status, assignee, priority)"
                        }
                    },
                    "required": ["id", "updates"]
                }
            }
        ]
    }))
}

/// Ping handler
async fn handle_ping() -> Result<Value> {
    Ok(Value::String("pong".to_string()))
}

/// Read context handler - fuzzy search across project
async fn handle_read_context(params: Option<Value>) -> Result<Value> {
    #[derive(Deserialize)]
    struct SearchParams {
        query: String,
        limit: Option<usize>,
        #[serde(default)]
        semantic: bool,
        #[serde(default)]
        mode: Option<String>,
        #[serde(default)]
        filters: Option<FilterParams>,
    }

    #[derive(Deserialize)]
    struct FilterParams {
        tags: Option<Vec<String>>,
        priority: Option<String>,
        assignee: Option<String>,
    }

    let params: SearchParams = params
        .ok_or_else(|| CueError::ValidationError("Missing params".to_string()))
        .and_then(|v| {
            serde_json::from_value(v)
                .map_err(|e| CueError::ValidationError(format!("Invalid params: {}", e)))
        })?;

    // Determine search mode: explicit mode > semantic flag > default hybrid
    let mode_str = if let Some(ref m) = params.mode {
        m.clone()
    } else if params.semantic {
        "semantic".to_string()
    } else {
        "hybrid".to_string()
    };

    tracing::info!(
        "read_context: query='{}', mode='{}'",
        params.query,
        mode_str
    );

    // Use CUE_WORKSPACE env var if set, otherwise use current directory
    let workspace = std::env::var("CUE_WORKSPACE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    // Convert MCP filters to core SearchFilters
    let search_filters = params.filters.map(|f| {
        cue_core::context::SearchFilters {
            tags: f.tags,
            priority: f.priority,
            assignee: f.assignee,
        }
    });

    // Use the new mode-aware search
    let search_mode = cue_core::context::SearchMode::parse(&mode_str);
    let results = cue_core::context::search_workspace_with_mode(
        &workspace,
        &params.query,
        search_mode,
        search_filters,
    )?;

    // Convert to simplified JSON response
    let limit = params.limit.unwrap_or(10);

    let json_results: Vec<Value> = results
        .into_iter()
        .take(limit)
        .map(|doc| {
            serde_json::json!({
                "path": doc.path,
                "hash": doc.hash,
                "tokens": doc.tokens,
                "anchors": doc.anchors.iter().take(3).map(|a| &a.header).collect::<Vec<_>>()
            })
        })
        .collect();

    Ok(serde_json::Value::Array(json_results))
}

/// Read document handler - read file with optional anchor
async fn handle_read_doc(params: Option<Value>) -> Result<Value> {
    #[derive(Deserialize)]
    struct ReadDocParams {
        path: String,
        #[allow(dead_code)]
        anchor: Option<String>,
    }

    let params: ReadDocParams = params
        .ok_or_else(|| CueError::ValidationError("Missing params".to_string()))
        .and_then(|v| {
            serde_json::from_value(v)
                .map_err(|e| CueError::ValidationError(format!("Invalid params: {}", e)))
        })?;

    // Validate path pattern
    if !params.path.ends_with(".md") {
        return Err(CueError::ValidationError(
            "Path must end with .md".to_string(),
        ));
    }

    // Use CUE_WORKSPACE env var if set, otherwise use current directory
    let workspace = std::env::var("CUE_WORKSPACE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    
    let file_path = workspace.join(&params.path);

    let doc = cue_core::parse_file(&file_path)?;

    serde_json::to_value(doc).map_err(CueError::JsonError)
}

/// List tasks handler - list task cards by status
async fn handle_list_tasks(params: Option<Value>) -> Result<Value> {
    #[derive(Deserialize)]
    struct ListTasksParams {
        status: Option<String>,
        assignee: Option<String>,
    }

    let params: ListTasksParams = if let Some(p) = params {
        serde_json::from_value(p)
            .map_err(|e| CueError::ValidationError(format!("Invalid params: {}", e)))?
    } else {
        ListTasksParams {
            status: None,
            assignee: None,
        }
    };

    // Use CUE_WORKSPACE env var if set, otherwise use current directory
    let workspace = std::env::var("CUE_WORKSPACE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    
    let tasks =
        cue_core::tasks::list_tasks(&workspace, params.status.as_deref(), params.assignee.as_deref())?;

    serde_json::to_value(tasks).map_err(CueError::JsonError)
}

/// Create task handler
async fn handle_create_task(params: Option<Value>) -> Result<Value> {
    #[derive(Deserialize)]
    struct CreateTaskParams {
        title: String,
    }

    let params: CreateTaskParams = params
        .ok_or_else(|| CueError::ValidationError("Missing params".to_string()))
        .and_then(|v| {
            serde_json::from_value(v)
                .map_err(|e| CueError::ValidationError(format!("Invalid params: {}", e)))
        })?;

    // Use CUE_WORKSPACE env var if set, otherwise use current directory
    let workspace = std::env::var("CUE_WORKSPACE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    
    let path = cue_core::tasks::create_task(&workspace, &params.title)?;

    // Return the created task doc
    let doc = cue_core::parse_file(&path)?;
    serde_json::to_value(doc).map_err(CueError::JsonError)
}

/// Update task handler - modify task frontmatter
async fn handle_update_task(params: Option<Value>) -> Result<Value> {
    #[derive(Deserialize)]
    struct UpdateTaskParams {
        id: String,
        updates: serde_json::Map<String, Value>,
    }

    let params: UpdateTaskParams = params
        .ok_or_else(|| CueError::ValidationError("Missing params".to_string()))
        .and_then(|v| {
            serde_json::from_value(v)
                .map_err(|e| CueError::ValidationError(format!("Invalid params: {}", e)))
        })?;

    // Validate ID pattern (6-char alphanumeric)
    if !params.id.chars().all(|c| c.is_ascii_alphanumeric()) || params.id.len() != 6 {
        return Err(CueError::ValidationError(
            "Task ID must be 6 alphanumeric characters".to_string(),
        ));
    }

    // Use CUE_WORKSPACE env var if set, otherwise use current directory
    let workspace = std::env::var("CUE_WORKSPACE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    
    let doc = cue_core::tasks::update_task(&workspace, &params.id, params.updates)?;

    serde_json::to_value(doc).map_err(CueError::JsonError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(1.into())),
            method: "ping".to_string(),
            params: None,
        };

        let resp = handle_request(req).await;
        assert!(resp.is_some());
        let response = resp.unwrap();
        assert!(response.result.is_some());
    }
}
