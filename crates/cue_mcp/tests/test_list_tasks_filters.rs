use assert_fs::prelude::*;
use cue_mcp::{handle_request, JsonRpcRequest};
use serde_json::{json, Value};
use std::env;

#[tokio::test]
async fn test_list_tasks_with_filters() {
    // Setup workspace
    let temp = assert_fs::TempDir::new().unwrap();
    let cards_dir = temp.child(".cuedeck/cards");
    cards_dir.create_dir_all().unwrap();

    // Create tasks
    // 1. High priority, tagged "backend"
    cards_dir.child("task1.md").write_str(r#"---
title: Task 1
status: todo
priority: high
tags: [backend, api]
created: 2024-01-01T00:00:00Z
---
# Task 1"#).unwrap();

    // 2. Low priority, tagged "frontend"
    cards_dir.child("task2.md").write_str(r#"---
title: Task 2
status: todo
priority: low
tags: [frontend, ui]
created: 2024-01-05T00:00:00Z
---
# Task 2"#).unwrap();

    // 3. Critical, tagged "backend", "db"
    cards_dir.child("task3.md").write_str(r#"---
title: Task 3
status: active
priority: critical
tags: [backend, db]
created: 2024-01-10T00:00:00Z
---
# Task 3"#).unwrap();

    // Set CUE_WORKSPACE
    env::set_var("CUE_WORKSPACE", temp.path());

    // Test 1: Filter by Tag (backend)
    {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "list_tasks",
                "arguments": {
                    "tags": ["backend"]
                }
            })),
        };

        let resp = handle_request(req).await.unwrap();
        let result = resp.result.unwrap();
        let content_str = result["content"][0]["text"].as_str().unwrap();
        let tasks: Vec<Value> = serde_json::from_str(content_str).unwrap();

        assert_eq!(tasks.len(), 2);
        let titles: Vec<&str> = tasks.iter()
            .map(|t| t["frontmatter"]["title"].as_str().unwrap())
            .collect();
        assert!(titles.contains(&"Task 1"));
        assert!(titles.contains(&"Task 3"));
    }

    // Test 2: Filter by Priority (critical)
    {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "list_tasks",
                "arguments": {
                    "priority": "critical"
                }
            })),
        };

        let resp = handle_request(req).await.unwrap();
        let result = resp.result.unwrap();
        let content_str = result["content"][0]["text"].as_str().unwrap();
        let tasks: Vec<Value> = serde_json::from_str(content_str).unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0]["frontmatter"]["title"], "Task 3");
    }

    // Test 3: Filter by Created Date (>2024-01-02)
    {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(3)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "list_tasks",
                "arguments": {
                    "created": ">2024-01-02"
                }
            })),
        };

        let resp = handle_request(req).await.unwrap();
        let result = resp.result.unwrap();
        let content_str = result["content"][0]["text"].as_str().unwrap();
        let tasks: Vec<Value> = serde_json::from_str(content_str).unwrap();

        assert_eq!(tasks.len(), 2); // Task 2 (Jan 5) and Task 3 (Jan 10)
        let titles: Vec<&str> = tasks.iter()
            .map(|t| t["frontmatter"]["title"].as_str().unwrap())
            .collect();
        assert!(titles.contains(&"Task 2"));
        assert!(titles.contains(&"Task 3"));
    }
}

#[tokio::test]
async fn test_list_tasks_invalid_date() {
    // Setup workspace
    let temp = assert_fs::TempDir::new().unwrap();
    let cards_dir = temp.child(".cuedeck/cards");
    cards_dir.create_dir_all().unwrap();
    env::set_var("CUE_WORKSPACE", temp.path());

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_tasks",
            "arguments": {
                "created": "invalid-date"
            }
        })),
    };

    let resp = handle_request(req).await.unwrap();
    
    // Expect error
    assert!(resp.result.is_none());
    let error = resp.error.unwrap();
    assert_eq!(error.code, -32700); // Parse error (from CueError::ParseError)
    assert!(error.message.contains("Invalid date format"));
}
