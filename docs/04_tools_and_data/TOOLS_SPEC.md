# Tools Specification

## MCP Tools (AI Facing)

### 1. `read_context`

- **Description**: Smart fuzzy/semantic search across project context headers and filenames with optional mode selection.
- **Complexity**:
  - Keyword: O(n log n) where n = total indexed headers
  - Semantic/Hybrid: O(n × d) where d = embedding dimension (384)
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
    },
    "mode": {
      "type": "string",
      "enum": ["keyword", "semantic", "hybrid"],
      "default": "hybrid",
      "description": "Search algorithm selection"
    },
      "default": false,
      "description": "DEPRECATED: Use mode='semantic' instead. Kept for backward compatibility."
    },
    "filters": {
      "type": "object",
      "properties": {
        "tags": { 
          "type": "array", 
          "items": { "type": "string" },
          "description": "Filter by tags (ANY match logic)" 
        },
        "priority": { 
          "type": "string",
          "description": "Filter by priority (case-insensitive)" 
        },
        "assignee": { 
          "type": "string", 
          "description": "Filter by assignee (case-insensitive)" 
        }
      }
    }
  },
  "required": ["query"]
}
```

**Mode Descriptions**:

- `keyword`: Fast text matching (filename + content tokens)
- `semantic`: AI embedding-based similarity (all-MiniLM-L6-v2)
- `hybrid` **(default)**: 70% semantic + 30% keyword weighting

**Mode Selection Priority**:

1. If `mode` is explicitly set → use that mode
2. Else if `semantic=true` → use "semantic" mode (backward compat)
3. Else → use "hybrid" (default)

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

**Example Requests**:

```json
// Hybrid search (default)
{
  "jsonrpc": "2.0",
  "method": "read_context",
  "params": {
    "query": "authentication",
    "limit": 10
  }
}

// Explicit keyword-only
{
  "jsonrpc": "2.0",
  "method": "read_context",
  "params": {
    "query": "login flow",
    "mode": "keyword"
  }
}

// Semantic search (new syntax)
{
  "jsonrpc": "2.0",
  "method": "read_context",
  "params": {
    "query": "concurrent programming",
    "mode": "semantic"
  }
}

// Filtered search
{
  "jsonrpc": "2.0",
  "method": "read_context",
  "params": {
    "query": "authentication",
    "filters": {
      "tags": ["auth", "security"],
      "priority": "high"
    }
  }
}
```

**Performance Expectations**:

- Keyword: < 100ms for 500 files
- Semantic (cold): 2-5s for first search (model download + embedding)
- Semantic (warm): 200-300ms (80%+ cache hit rate)
- Hybrid: Similar to semantic (cache-dependent)

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
    },
    "tags": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Filter by tags (ANY match logic)"
    },
    "priority": {
      "type": "string",
      "enum": ["low", "medium", "high", "critical"],
      "description": "Filter by priority"
    },
    "created": {
      "type": "string",
      "description": "Filter by creation date (e.g. '2024-01-01', '>2w')"
    },
    "updated": {
      "type": "string",
      "description": "Filter by update date (e.g. '2024-01-01', '>2w')"
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

### 4. `create_task`

- **Description**: Create a new task card with rich metadata and dependency tracking.
- **Complexity**: O(1) for task creation + O(d) for dependency validation where d = number of dependencies
- **Input Schema**:

```json
{
  "type": "object",
  "properties": {
    "title": {
      "type": "string",
      "description": "Task title",
      "minLength": 1,
      "maxLength": 200
    },
    "tags": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Optional tags for categorization"
    },
    "priority": {
      "type": "string",
      "enum": ["low", "medium", "high", "critical"],
      "default": "medium",
      "description": "Task priority level"
    },
    "assignee": {
      "type": "string",
      "description": "Person assigned to this task"
    },
    "depends_on": {
      "type": "array",
      "items": {
        "type": "string",
        "pattern": "^[a-z0-9]{6}$"
      },
      "description": "Task IDs this task depends on"
    }
  },
  "required": ["title"]
}
```

- **Output Schema**:

```json
{
  "type": "object",
  "properties": {
    "id": { "type": "string", "pattern": "^[a-z0-9]{6}$" },
    "path": { "type": "string" },
    "metadata": {
      "type": "object",
      "properties": {
        "title": { "type": "string" },
        "status": { "type": "string" },
        "priority": { "type": "string" },
        "assignee": { "type": ["string", "null"] },
        "tags": { "type": "array", "items": { "type": "string" } },
        "depends_on": { "type": "array", "items": { "type": "string" } },
        "created": { "type": "string", "format": "date-time" }
      }
    }
  },
  "required": ["id", "path", "metadata"]
}
```

**Example Request**:

```json
{
  "title": "Implement login API",
  "tags": ["auth", "backend"],
  "priority": "high",
  "assignee": "@developer",
  "depends_on": ["abc123", "def456"]
}
```

**Example Response**:

```json
{
  "id": "xyz789",
  "path": ".cuedeck/cards/xyz789.md",
  "metadata": {
    "title": "Implement login API",
    "status": "todo",
    "priority": "high",
    "assignee": "@developer",
    "tags": ["auth", "backend"],
    "depends_on": ["abc123", "def456"],
    "created": "2026-01-02T10:43:00Z"
  }
}
```

**Error Responses**:

- `DependencyNotFound`: If a task ID in `depends_on` does not exist
- `CircularDependency`: If adding dependencies would create a cycle

### 5. `get_task_dependencies`

- **Description**: Get dependencies for a task (forward or reverse).
- **Complexity**: O(n) where n = total tasks (builds graph), O(d) for query where d = direct dependencies
- **Input Schema**:

```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "pattern": "^[a-z0-9]{6}$",
      "description": "Task ID to query"
    },
    "reverse": {
      "type": "boolean",
      "default": false,
      "description": "If true, get dependents (tasks that depend on this task)"
    }
  },
  "required": ["id"]
}
```

- **Output Schema**:

```json
{
  "type": "object",
  "properties": {
    "task_id": { "type": "string" },
    "type": { "type": "string", "enum": ["dependencies", "dependents"] },
    "count": { "type": "integer" },
    "tasks": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": { "type": "string" },
          "title": { "type": "string" },
          "status": { "type": "string" }
        }
      }
    }
  },
  "required": ["task_id", "type", "count", "tasks"]
}
```

**Example Request (Dependencies)**:

```json
{
  "id": "xyz789",
  "reverse": false
}
```

**Example Response**:

```json
{
  "task_id": "xyz789",
  "type": "dependencies",
  "count": 2,
  "tasks": [
    { "id": "abc123", "title": "Setup auth framework", "status": "done" },
    { "id": "def456", "title": "Create user database", "status": "active" }
  ]
}
```

**Example Request (Dependents - Reverse)**:

```json
{
  "id": "xyz789",
  "reverse": true
}
```

**Example Response**:

```json
{
  "task_id": "xyz789",
  "type": "dependents",
  "count": 1,
  "tasks": [
    { "id": "ghi012", "title": "Add login UI", "status": "todo" }
  ]
}
```

### 6. `validate_task_graph`

- **Description**: Validate task dependency graph for circular dependencies.
- **Complexity**: O(n + e) where n = tasks, e = dependency edges (cycle detection via DFS)
- **Input Schema**:

```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "pattern": "^[a-z0-9]{6}$",
      "description": "Optional: validate specific task only"
    }
  }
}
```

- **Output Schema**:

```json
{
  "type": "object",
  "properties": {
    "valid": { "type": "boolean" },
    "task_id": { "type": "string" },
    "message": { "type": "string" },
    "error": { "type": "string" }
  },
  "required": ["valid"]
}
```

**Example Request (Full Graph)**:

```json
{}
```

**Example Response (Valid)**:

```json
{
  "valid": true,
  "message": "All task dependencies are valid (no circular dependencies)"
}
```

**Example Response (Invalid - Cycle Detected)**:

```json
{
  "valid": false,
  "error": "Circular dependency detected: abc123 → def456 → ghi789 → abc123"
}
```

**Example Request (Specific Task)**:

```json
{
  "id": "xyz789"
}
```

**Example Response**:

```json
{
  "valid": true,
  "task_id": "xyz789",
  "message": "Task dependencies are valid"
}
```

### 7. `update_task`

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
