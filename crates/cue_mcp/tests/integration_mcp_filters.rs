use assert_fs::prelude::*;
use cue_mcp::{handle_request, JsonRpcRequest};
use serde_json::{json, Value};
use std::env;

#[tokio::test]
async fn test_integration_create_and_filter() {
    // 1. Setup Request/Response Helper
    async fn call_tool(name: &str, args: Value) -> Value {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": name,
                "arguments": args
            })),
        };
        
        let resp = handle_request(req).await.expect("Request failed");
        assert!(resp.error.is_none(), "Tool call returned error: {:?}", resp.error);
        
        let content = resp.result.unwrap()["content"][0]["text"].as_str().unwrap().to_string();
        serde_json::from_str(&content).expect("Failed to parse tool response")
    }

    // 2. Setup Workspace
    let temp = assert_fs::TempDir::new().unwrap();
    let cards_dir = temp.child(".cuedeck/cards");
    cards_dir.create_dir_all().unwrap();
    env::set_var("CUE_WORKSPACE", temp.path());

    // 3. Create Task A (High Priority, Tag: backend)
    let _task_a = call_tool("create_task", json!({
        "title": "Task A",
        "priority": "high",
        "tags": ["backend"]
    })).await;

    // 4. Create Task B (Low Priority, Tag: frontend)
    let _task_b = call_tool("create_task", json!({
        "title": "Task B",
        "priority": "low",
        "tags": ["frontend"]
    })).await;

    // 5. Verify Filter by Priority: High
    let tasks_high = call_tool("list_tasks", json!({
        "priority": "high"
    })).await;
    let tasks_arr = tasks_high.as_array().unwrap();
    assert_eq!(tasks_arr.len(), 1);
    assert_eq!(tasks_arr[0]["frontmatter"]["title"], "Task A");

    // 6. Verify Filter by Tag: frontend
    let tasks_frontend = call_tool("list_tasks", json!({
        "tags": ["frontend"]
    })).await;
    let tasks_arr = tasks_frontend.as_array().unwrap();
    assert_eq!(tasks_arr.len(), 1);
    assert_eq!(tasks_arr[0]["frontmatter"]["title"], "Task B");

     // 7. Verify Filter by Tag: backend OR frontend (Should get both)
     let tasks_both = call_tool("list_tasks", json!({
        "tags": ["backend", "frontend"]
    })).await;
    let tasks_arr = tasks_both.as_array().unwrap();
    assert_eq!(tasks_arr.len(), 2);
}
