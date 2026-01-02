use serde_json::json;

#[tokio::test]
async fn test_read_context_pagination() {
    use cue_mcp::{handle_request, JsonRpcRequest};
    
    // Create test request with pagination
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "read_context",
            "arguments": {
                "query": "test",
                "limit": 5
            }
        })),
    };
    
    let resp = handle_request(req).await;
    assert!(resp.is_some());
    
    let response = resp.unwrap();
    assert!(response.error.is_none(), "Should not have error");
    
    if let Some(result) = response.result {
        let content = result.get("content").expect("Should have content field");
        let text = content[0].get("text").and_then(|v| v.as_str()).expect("Should have text");
        let parsed: serde_json::Value = serde_json::from_str(text).expect("Should parse JSON");
        
        // Verify pagination fields exist
        assert!(parsed.get("results").is_some(), "Should have results field");
        assert!(parsed.get("total_count").is_some(), "Should have total_count field");
        assert!(parsed.get("next_cursor").is_some(), "Should have next_cursor field");
        assert!(parsed.get("has_more").is_some(), "Should have has_more field");
        
        let total = parsed["total_count"].as_u64().unwrap();
        let results_len = parsed["results"].as_array().unwrap().len();
        
        // If total > 5, we should have a cursor
        if total > 5 {
            assert!(parsed["next_cursor"].is_string(), "Should have cursor when more results exist");
            assert_eq!(parsed["has_more"].as_bool().unwrap(), true);
        }
        
        // Results should be limited to 5
        assert!(results_len <= 5, "Should return at most 5 results");
    } else {
        panic!("Should have result");
    }
}

#[tokio::test]
async fn test_read_context_with_cursor() {
    use cue_mcp::{handle_request, JsonRpcRequest};
    
    // First request - get page 1
    let req1 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "read_context",
            "arguments": {
                "query": "test",
                "limit": 3
            }
        })),
    };
    
    let resp1 = handle_request(req1).await;
    assert!(resp1.is_some());
    
    let response1 = resp1.unwrap();
    if let Some(result1) = response1.result {
        let content1 = result1.get("content").unwrap();
        let text1 = content1[0].get("text").unwrap().as_str().unwrap();
        let parsed1: serde_json::Value = serde_json::from_str(text1).unwrap();
        
        let total = parsed1["total_count"].as_u64().unwrap();
        
        // Only test cursor if there are enough results
        if total > 3 {
            let cursor = parsed1["next_cursor"].as_str().expect("Should have cursor");
            
            // Second request - use cursor to get page 2
            let req2 = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(2)),
                method: "tools/call".to_string(),
                params: Some(json!({
                    "name": "read_context",
                    "arguments": {
                        "query": "test",
                        "limit": 3,
                        "cursor": cursor
                    }
                })),
            };
            
            let resp2 = handle_request(req2).await;
            assert!(resp2.is_some());
            
            let response2 = resp2.unwrap();
            if let Some(result2) = response2.result {
                let content2 = result2.get("content").unwrap();
                let text2 = content2[0].get("text").unwrap().as_str().unwrap();
                let parsed2: serde_json::Value = serde_json::from_str(text2).unwrap();
                
                // Verify total count is consistent
                assert_eq!(parsed2["total_count"].as_u64().unwrap(), total);
                
                // Get result paths from both pages
                let page1_results = parsed1["results"].as_array().unwrap();
                let page2_results = parsed2["results"].as_array().unwrap();
                
                // Verify different results (no overlap)
                if !page1_results.is_empty() && !page2_results.is_empty() {
                    let page1_paths: Vec<&str> = page1_results
                        .iter()
                        .filter_map(|r| r.get("path").and_then(|p| p.as_str()))
                        .collect();
                    let page2_paths: Vec<&str> = page2_results
                        .iter()
                        .filter_map(|r| r.get("path").and_then(|p| p.as_str()))
                        .collect();
                    
                    for p2 in &page2_paths {
                        assert!(!page1_paths.contains(p2), "Pages should not contain same documents");
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_read_context_limit_clamping() {
    use cue_mcp::{handle_request, JsonRpcRequest};
    
    // Request with limit > 50 (should be clamped to 50)
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "read_context",
            "arguments": {
                "query": "test",
                "limit": 100
            }
        })),
    };
    
    let resp = handle_request(req).await;
    assert!(resp.is_some());
    
    let response = resp.unwrap();
    if let Some(result) = response.result {
        let content = result.get("content").unwrap();
        let text = content[0].get("text").unwrap().as_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
        
        let results = parsed["results"].as_array().unwrap();
        assert!(results.len() <= 50, "Should clamp limit to max 50");
    }
}

#[tokio::test]
async fn test_read_context_invalid_cursor() {
    use cue_mcp::{handle_request, JsonRpcRequest};
    
    // Request with invalid cursor
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "read_context",
            "arguments": {
                "query": "test",
                "limit": 5,
                "cursor": "invalid_base64!!!"
            }
        })),
    };
    
    let resp = handle_request(req).await;
    assert!(resp.is_some());
    
    let response = resp.unwrap();
    // Should return an error for invalid cursor
    assert!(response.error.is_some(), "Should have error for invalid cursor");
}
