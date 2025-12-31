# Tools Specification

## MCP Tools (AI Facing)

### 1. `read_context`

- **Description**: Smart fuzzy search across the project context headers and filenames.
- **Complexity**: O(n log n) where n = total indexed headers
- **Input Schema**:

```json
{
  "type": "object",
  "properties": {
    "query": { 
      "type": "string", 
      "description": "Keywords to match (e.g. 'auth flow')",
      "minLength": 1,
      "maxLength": 200
    },
    "limit": { 
      "type": "integer", 
      "default": 5,
      "minimum": 1,
      "maximum": 50
    }
  },
  "required": ["query"]
}
```

- **Output Schema**:

```json
{
  "type": "array",
  "items": {
    "type": "object",
    "properties": {
      "path": { "type": "string", "description": "Relative path from workspace root" },
      "hash": { "type": "string", "description": "SHA256 hash of content" },
      "tokens": { "type": "integer", "description": "Token count" },
      "anchors": { 
        "type": "array", 
        "items": { "type": "string" },
        "description": "Top 3 headers/anchors" 
      }
    },
    "required": ["path", "hash", "tokens"]
  }
}
```

**Example Output**:

```json
[
  { 
    "path": "docs/auth.md", 
    "hash": "a1b2c3d4...", 
    "tokens": 450, 
    "anchors": ["Login Flow", "OAuth Configuration"] 
  },
  { 
    "path": "crates/cue_core/src/auth.rs", 
    "hash": "e5f6g7h8...", 
    "tokens": 1200, 
    "anchors": ["VerifyToken"] 
  }
]
```

### 2. `read_doc`

- **Description**: Reads a specific document or a section of it (Inclusion Mode).
- **Complexity**: O(1) for cached reads, O(n) for cold reads where n = file size
- **Input Schema**:

```json
{
  "type": "object",
  "properties": {
    "path": { 
      "type": "string", 
      "description": "Relative path to file",
      "pattern": "^[^.][a-zA-Z0-9_/.\\-]+\\.md$"
    },
    "anchor": { 
      "type": "string", 
      "description": "Optional Header name to slice from",
      "maxLength": 100
    }
  },
  "required": ["path"]
}
```

- **Output Schema**:

```json
{
  "type": "object",
  "properties": {
    "path": { 
      "type": "string", 
      "description": "Normalized file path" 
    },
    "content": { 
      "type": "string", 
      "description": "Markdown content (trimmed)" 
    },
    "anchor": { 
      "type": ["string", "null"], 
      "description": "Requested anchor if any" 
    },
    "tokens": { 
      "type": "integer", 
      "description": "Estimated token count" 
    },
    "hash": { 
      "type": "string", 
      "description": "SHA256 hash of content for cache validation" 
    },
    "cached": { 
      "type": "boolean", 
      "description": "Whether served from cache" 
    },
    "line_range": {
      "type": ["object", "null"],
      "properties": {
        "start": { "type": "integer", "description": "Start line (1-indexed)" },
        "end": { "type": "integer", "description": "End line (inclusive)" }
      },
      "description": "Line range if anchor was specified"
    }
  },
  "required": ["path", "content", "tokens"]
}
```

**Example Output (Full File)**:

```json
{
  "path": "docs/api.md",
  "content": "# API Documentation\n\nThis document...",
  "anchor": null,
  "tokens": 1523,
  "hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "cached": true,
  "line_range": null
}
```

**Example Output (With Anchor)**:

```json
{
  "path": "docs/api.md",
  "content": "## Error Codes\n\nThe following error codes...",
  "anchor": "Error Codes",
  "tokens": 342,
  "hash": "a7f1d92...",
  "cached": false,
  "line_range": { "start": 45, "end": 89 }
}
```

### 3. `list_tasks`

- **Description**: Get tasks filtered by status.
- **Complexity**: O(n) where n = total task cards
- **Input Schema**:

```json
{
  "type": "object",
  "properties": {
    "status": { 
      "type": "string", 
      "enum": ["todo", "active", "done", "archived"],
      "description": "Filter tasks by status"
    },
    "assignee": {
      "type": "string",
      "description": "Optional filter by assignee name"
    }
  }
}
```

- **Output Schema**:

```json
{
  "type": "array",
  "items": {
    "type": "object",
    "properties": {
      "id": { "type": "string", "pattern": "^[a-z0-9]{6}$" },
      "title": { "type": "string" },
      "priority": { "type": "string", "enum": ["low", "medium", "high", "critical"] },
      "assignee": { "type": ["string", "null"] }
    }
  }
}
```

### 4. `update_task`

- **Description**: Modify a task card's frontmatter.
- **Complexity**: O(1) for metadata update + O(n) for file write where n = card size
- **Input Schema**:

```json
{
  "type": "object",
  "properties": {
    "id": { 
      "type": "string", 
      "pattern": "^[a-z0-9]{6}$",
      "description": "6-char alphanumeric task ID"
    },
    "updates": {
      "type": "object",
      "properties": {
        "status": { "type": "string", "enum": ["todo", "active", "done", "archived"] },
        "assignee": { "type": "string", "maxLength": 50 },
        "priority": { "type": "string", "enum": ["low", "medium", "high", "critical"] },
        "notes": { "type": "string", "maxLength": 500 }
      },
      "minProperties": 1
    }
  },
  "required": ["id", "updates"]
}
```

- **Output Schema**:

```json
{
  "type": "object",
  "properties": {
    "id": { "type": "string", "pattern": "^[a-z0-9]{6}$" },
    "title": { "type": "string" },
    "status": { "type": "string", "enum": ["todo", "active", "done", "archived"] },
    "assignee": { "type": ["string", "null"] },
    "priority": { "type": "string", "enum": ["low", "medium", "high", "critical"] },
    "updated_at": { "type": "string", "format": "date-time" },
    "updated_fields": { "type": "array", "items": { "type": "string" } }
  },
  "required": ["id", "title", "status", "updated_at", "updated_fields"]
}
```

**Example Output**:

```json
{
  "id": "2a9f1x",
  "title": "Implement Login",
  "status": "active",
  "assignee": "user",
  "priority": "high",
  "updated_at": "2025-12-31T10:30:00Z",
  "updated_fields": ["status", "priority"]
}
```

### 5. Error Responses

All tools return standard JSON-RPC 2.0 Errors on failure.

#### Example: File Not Found

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": 1001,
    "message": "File Not Found",
    "data": {
      "path": "docs/missing.md",
      "suggestion": "Did you mean 'docs/existing.md'?"
    }
  }
}
```

#### Example: Token Limit Exceeded

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": 1003,
    "message": "Token Limit Exceeded",
    "data": {
      "current_size": 35000,
      "limit": 32000,
      "action": "Pruned 'Low Priority' nodes."
    }
  }
}
```

#### Example: Circular Dependency

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": 1002,
    "message": "Cycle Detected",
    "data": {
      "cycle_path": ["cards/task-a.md", "docs/api.md", "cards/task-a.md"],
      "suggestion": "Remove reference from cards/task-a.md"
    }
  }
}
```

#### Example: Stale Cache

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "error": {
    "code": 1006,
    "message": "Stale Cache",
    "data": {
      "path": "docs/api.md",
      "expected_hash": "e3b0c44...",
      "actual_hash": "a7f1d92...",
      "recovery": "Run 'cue clean' to rebuild cache"
    }
  }
}
```

#### Example: Lock Error

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": 1007,
    "message": "Lock Error",
    "data": {
      "lock_file": ".cuedeck/.cache/lock",
      "holder_pid": 12345,
      "suggestion": "Close other CueDeck instances or remove stale lock"
    }
  }
}
```

### 6. Rate Limiting & Quotas

MCP tools implement rate limiting to prevent resource exhaustion:

| Tool | Rate Limit | Burst Limit | Window |
| :--- | :--- | :--- | :--- |
| `read_context` | 10 req/min | 15 req | 60s |
| `read_doc` | 30 req/min | 50 req | 60s |
| `list_tasks` | 20 req/min | 30 req | 60s |
| `update_task` | 10 req/min | 15 req | 60s |

**Rate Limit Error Response**:

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "error": {
    "code": 429,
    "message": "Rate Limit Exceeded",
    "data": {
      "retry_after_seconds": 45,
      "limit": 10,
      "window_seconds": 60,
      "current_count": 15
    }
  }
}
```

**Retry Strategy**:

Clients should implement exponential backoff:

```python
# Pseudo-code
max_retries = 3
base_delay = 1.0  # seconds

for attempt in range(max_retries):
    response = call_mcp_tool(request)
    if response.error and response.error.code == 429:
        delay = base_delay * (2 ** attempt)  # Exponential backoff
        sleep(delay)
        continue
    return response
```

## CLI Tools (User Facing)

### `cue init`

- **Purpose**: Initialize CueDeck workspace.
- **Creates**: `.cuedeck/{cards,docs,config.toml}`, updates `.gitignore`.
- **Flags**: `--force` to overwrite existing configuration.

### `cue scene`

- **Purpose**: Generate context snapshot for AI.
- **Flags**:
  - `--dry-run`, `-d`: Output to stdout instead of clipboard.
  - `--token-limit <N>`: Override configured token limit.
- **Output**: `SCENE.md` written to `.cuedeck/` and copied to clipboard.

### `cue open [query]`

- **Purpose**: Interactive fuzzy finder (TUI).
- **Library**: `skim` crate.
- **Behavior**: `Enter` opens file in `$EDITOR`.

### `cue watch`

- **Purpose**: Real-time file monitoring daemon.
- **Library**: `notify` crate.
- **Behavior**: Debounces events (500ms), auto-updates clipboard via `arboard`.

### `cue doctor`

- **Purpose**: Health check and diagnostics.
- **Library**: `miette` for beautiful error reports.
- **Checks**: Config syntax, YAML frontmatter, dead links, circular deps.

### `cue card new <TITLE>`

- **Purpose**: Create new task card.
- **Behavior**: Generates 6-char Hash ID from timestamp + title.
- **Output**: `.cuedeck/cards/<id>.md` with populated frontmatter.

### `cue clean`

- **Purpose**: Clear cache for fresh start.
- **Action**: Removes `.cuedeck/.cache/` directory.

### `cue upgrade`

- **Purpose**: Self-update to latest version.
- **Behavior**: Downloads and replaces binary from release server.

### `cue mcp`

- **Purpose**: Start MCP Server for AI integration.
- **Transport**: JSON-RPC 2.0 over `stdio`.
- **Critical**: `stdout` reserved for responses; logs go to `stderr`.

## Tool Complexity Constraints

| Tool | Time Complexity | Space Complexity | Max Response Time |
| :--- | :--- | :--- | :--- |
| `read_context` | O(n log n) | O(n) | 100ms |
| `read_doc` | O(1) cached, O(n) cold | O(n) | 50ms |
| `list_tasks` | O(n) | O(n) | 20ms |
| `update_task` | O(1) + O(n) write | O(n) | 30ms |

> **Note**: `n` = number of indexed items (headers, files, tasks) or file size in bytes.

## Batch Operations

For performance, agents should:

- **Avoid**: Calling `read_doc` in a loop for 10+ files.
- **Prefer**: Use `read_context` to identify relevant files first, then selective `read_doc` calls.
- **Pattern**: `read_context("authentication")` → `read_doc(top_3_results)` → synthesize answer.

---
**Related Docs**: [API_DOCUMENTATION.md](./API_DOCUMENTATION.md), [CLI_REFERENCE.md](./CLI_REFERENCE.md), [EXAMPLES.md](../03_agent_design/EXAMPLES.md)
