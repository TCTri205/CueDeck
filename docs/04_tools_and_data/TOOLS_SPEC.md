# Tools Specification

## MCP Tools (AI Facing)

### 1. `read_context`

- **Description**: Smart fuzzy search across the project context headers and filenames.
- **Input Schema**:

```json
{
  "type": "object",
  "properties": {
    "query": { "type": "string", "description": "Keywords to match (e.g. 'auth flow')" },
    "limit": { "type": "integer", "default": 5 }
  },
  "required": ["query"]
}
```

- **Output Schema**:

```json
[
  { "file": "docs/auth.md", "header": "Login Flow", "score": 0.95 },
  { "file": "crates/cue_core/src/auth.rs", "header": "VerifyToken", "score": 0.82 }
]
```

### 2. `read_doc`

- **Description**: Reads a specific document or a section of it (Inclusion Mode).
- **Input Schema**:

```json
{
  "type": "object",
  "properties": {
    "path": { "type": "string", "description": "Relative path to file" },
    "anchor": { "type": "string", "description": "Optional Header name to slice from" }
  },
  "required": ["path"]
}
```

- **Output**: Pure Markdown string (trimmed).

### 3. `list_tasks`

- **Description**: Get tasks filtered by status.
- **Input Schema**:

```json
{ "status": "active" } // enum: ['todo', 'active', 'done', 'archived']
```

### 4. `update_task`

- **Description**: Modify a task card's frontmatter.
- **Input Schema**:

```json
{
  "id": "2a9f1x",
  "updates": {
    "status": "done",
    "assignee": "NewUser",
    "notes": "Fixed in PR #42"
  }
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

## CLI Tools (User Facing)

- **`cue init`**:
  - Creates `.cuedeck/{cards,docs,.cache,config.toml}`.
  - Adds `.cuedeck/.cache` to `.gitignore`.
- **`cue open [query]`**:
  - Launches `skim` TUI.
  - `Enter` -> Opens `$EDITOR`.
- **`cue scene [--dry-run]`**:
  - Builds the context.
  - `--dry-run`: Prints to stdout instead of clipboard.

---
**Related Docs**: [API_DOCUMENTATION.md](./API_DOCUMENTATION.md), [EXAMPLES.md](../03_agent_design/EXAMPLES.md), [KNOWLEDGE_BASE_STRUCTURE.md](./KNOWLEDGE_BASE_STRUCTURE.md)
