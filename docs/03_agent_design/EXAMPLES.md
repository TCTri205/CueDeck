# Agent Interaction Examples

This document provides concrete request/response pairs for each MCP tool.

## Quick Reference

| Tool | Purpose | Use Case |
| :--- | :--- | :--- |
| `read_context` | Fuzzy search by keywords | "Find auth-related code" |
| `read_doc` | Get specific file/section | "Read error codes section" |
| `list_tasks` | Show active work items | "What am I working on?" |
| `update_task` | Modify task status/notes | "Mark login task as done" |

---

## 1. `read_context` - Fuzzy Search

### Request (Fuzzy Search)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "read_context",
  "params": { "query": "authentication flow" }
}
```

### Response (Context Results)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": [
    { "file": "docs/auth.md", "header": "Login Sequence", "score": 0.92, "snippet": "User submits credentials..." },
    { "file": "crates/cue_core/src/auth.rs", "header": "verify_token", "score": 0.78, "snippet": "pub fn verify_token(token: &str)..." }
  ]
}
```

### Request (With Limit)

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "read_context",
  "params": { "query": "error handling", "limit": 3 }
}
```

---

## 2. `read_doc` - Granular Read

### Request (Full File)

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "read_doc",
  "params": { "path": "docs/api.md" }
}
```

### Request (Specific Anchor)

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "read_doc",
  "params": { "path": "docs/api.md", "anchor": "Error Codes" }
}
```

### Response (Success - Anchor Mode)

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": "## Error Codes\n\n| Code | Message |\n|---|---|\n| 1001 | File Not Found |\n...",
    "tokens": 250
  }
}
```

### Response (File Not Found - Error 1001)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": 1001,
    "message": "File not found: docs/missing.md",
    "data": { "suggestion": "Did you mean docs/api.md?" }
  }
}
```

---

## 3. `list_tasks` - Task Listing

### Request (Active Tasks Only)

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "list_tasks",
  "params": { "status": "active" }
}
```

### Response (Task List)

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": [
    { "id": "2a9f1x", "title": "Implement Login", "priority": "high", "assignee": "dev" },
    { "id": "8b2c4z", "title": "Write Unit Tests", "priority": "medium", "assignee": null }
  ]
}
```

### Request (All Tasks with Filter)

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "list_tasks",
  "params": { "status": "all", "assignee": "dev" }
}
```

---

## 4. `update_task` - Task Modification

### Request (Mark as Done)

```json
{
  "jsonrpc": "2.0",
  "id": 8,
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
  "id": 8,
  "result": { "success": true, "message": "Card 2a9f1x updated." }
}
```

### Request (Add Notes Only)

```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "update_task",
  "params": {
    "id": "8b2c4z",
    "updates": { "notes": "Blocked on API changes" }
  }
}
```

---

## 5. Common Error Responses

### Error 1001: File Not Found

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "error": {
    "code": 1001,
    "message": "File not found",
    "data": { "path": "docs/nonexistent.md" }
  }
}
```

### Error 1002: Cyclic Dependency

```json
{
  "jsonrpc": "2.0",
  "id": 11,
  "error": {
    "code": 1002,
    "message": "Cyclic dependency detected",
    "data": { "cycle": "cards/a.md -> docs/b.md -> cards/a.md" }
  }
}
```

### Error 1003: Token Limit Exceeded

```json
{
  "jsonrpc": "2.0",
  "id": 12,
  "error": {
    "code": 1003,
    "message": "Token limit exceeded",
    "data": { "requested": 45000, "limit": 32000 }
  }
}
```

### Error -32601: Unknown Method

```json
{
  "jsonrpc": "2.0",
  "id": 13,
  "error": {
    "code": -32601,
    "message": "Method not found: unknown_method"
  }
}
```

---

## 6. Workflow Examples

### Typical Agent Workflow

```text
1. Agent receives user task: "Fix login bug"

2. Agent searches for context:
   → read_context(query="login")
   ← Returns: auth.rs, login.md, user.rs

3. Agent reads specific section:
   → read_doc(path="docs/auth.md", anchor="Login Flow")
   ← Returns: Detailed login flow documentation

4. Agent lists current work:
   → list_tasks(status="active")
   ← Returns: "Fix Login Bug" task found

5. Agent completes work and updates task:
   → update_task(id="abc123", updates={status: "done"})
   ← Returns: success confirmation
```

### Error Recovery Pattern

```text
1. Agent attempts to read missing file:
   → read_doc(path="docs/missing.md")
   ← Error 1001: File not found, suggestion: "Did you mean docs/api.md?"

2. Agent uses suggestion:
   → read_doc(path="docs/api.md")
   ← Success: File content returned
```

---

## 7. Best Practices for Agents

### DO ✅

- Use `read_context` first to discover relevant files
- Request specific anchors instead of full files
- Update task notes with progress
- Handle errors gracefully

### DON'T ❌

- Request files outside workspace sandbox
- Ignore error suggestions in response data
- Make excessive requests (rate limit aware)
- Assume file paths without verification

## 8. Failure Scenarios & Recovery

### Scenario 1: Stale Cache Detection

```text
1. Agent requests file:
   → read_doc(path="docs/api.md")
   ← Error 1006: Stale cache (hash mismatch)

2. Agent triggers cache refresh:
   → CLI fallback: cue clean && cue scene
   ← Success: Fresh cache rebuilt

3. Agent retries original request:
   → read_doc(path="docs/api.md")
   ← Success: Current content returned
```

### Scenario 2: Circular Dependency Recovery

```text
1. User creates problematic ref:
   Card A refs → Doc B refs → Card A

2. Agent detects cycle on next operation:
   → cue scene
   ← Error 1002: Cycle detected [A→B→A]

3. Agent suggests fix:
   "Remove 'refs: A' from docs/B.md to break cycle"

4. User fixes, agent verifies:
   → cue doctor
   ← Success: No cycles found
```

### Scenario 3: Token Budget Exceeded

```text
1. Agent builds large scene:
   → cue scene
   ← Error 1003: Token limit exceeded (45K > 32K)

2. Agent tries with increased limit:
   → cue scene --limit=50000
   ← Warning: Truncated at 48K tokens

3. Agent suggests optimization:
   "Use anchor refs instead of full file includes"
   Example: Change "docs/api.md" → "docs/api.md#Login"
```

### Scenario 4: Missing File Graceful Fallback

```text
1. Agent attempts read:
   → read_doc(path="docs/deployment.md")
   ← Error 1001: File not found
   ← Data: {suggestion: "docs/deploy-guide.md"}

2. Agent uses suggestion:
   → read_doc(path="docs/deploy-guide.md")
   ← Success: Content returned

3. Agent informs user:
   "Note: Used deploy-guide.md instead of deployment.md (file not found)"
```

### Scenario 5: Lock Contention Recovery

```text
1. Agent calls update during migration:
   → update_task(id="abc", updates={status: "done"})
   ← Error 1007: Lock error (migration in progress)

2. Agent waits and retries:
   WAIT 1s
   → update_task(id="abc", updates={status: "done"})
   ← Success: Task updated

3. If still locked after 2 retries:
   Report to user: "Task update failed: System locked. Try again in 30s."
```

## 9. Edge Case Handling Matrix

| Edge Case | Detection | Recovery | Prevention |
| :--- | :--- | :--- | :--- |
| **Empty workspace** | No .cuedeck/ dir | Run `cue init` | Document setup in README |
| **No active cards** | `list_tasks()` returns [] | Prompt user to create card | Default card on init |
| **Malformed frontmatter** | Parse error | Skip file + log warning | Validate on save |
| **Anchor not found** | Search fails | Return full file | Suggest closest match |
| **Rate limit hit** | Error -32000 | Exponential backoff | Batch requests |

---
**Related Docs**: [TOOLS_SPEC.md](../04_tools_and_data/TOOLS_SPEC.md), [API_DOCUMENTATION.md](../04_tools_and_data/API_DOCUMENTATION.md), [PROMPTS_AND_INSTRUCTIONS.md](./PROMPTS_AND_INSTRUCTIONS.md)
