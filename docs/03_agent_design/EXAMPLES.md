# Agent Interaction Examples

This document provides concrete request/response pairs for each MCP tool.

---

## 1. `read_context` - Fuzzy Search

### Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "read_context",
  "params": { "query": "authentication flow" }
}
```

### Response (Success)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": [
    { "file": "docs/auth.md", "anchor": "Login Sequence", "score": 0.92, "snippet": "User submits credentials..." },
    { "file": "crates/cue_core/src/auth.rs", "anchor": "verify_token", "score": 0.78, "snippet": "pub fn verify_token(token: &str)..." }
  ]
}
```

---

## 2. `read_doc` - Granular Read

### Request (Full File)

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "read_doc",
  "params": { "path": "docs/api.md" }
}
```

### Request (Specific Anchor)

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "read_doc",
  "params": { "path": "docs/api.md", "anchor": "Error Codes" }
}
```

### Response (Success - Anchor Mode)

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": "## Error Codes\n\n| Code | Message |\n|---|---|\n| 1001 | File Not Found |\n...",
    "token_count": 250
  }
}
```

---

## 3. `list_tasks` - Task Listing

### Request

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "list_tasks",
  "params": { "status": "active" }
}
```

### Response (Success)

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": [
    { "id": "2a9f1x", "title": "Implement Login", "priority": "high", "assignee": "dev" },
    { "id": "8b2c4z", "title": "Write Unit Tests", "priority": "medium", "assignee": null }
  ]
}
```

---

## 4. `update_task` - Task Modification

### Request

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "update_task",
  "params": {
    "id": "2a9f1x",
    "updates": { "status": "done", "notes": "Completed in PR #42" }
  }
}
```

### Response (Success)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": { "success": true, "message": "Card 2a9f1x updated." }
}
```

---
**Related Docs**: [TOOLS_SPEC.md](../04_tools_and_data/TOOLS_SPEC.md), [API_DOCUMENTATION.md](../04_tools_and_data/API_DOCUMENTATION.md)
