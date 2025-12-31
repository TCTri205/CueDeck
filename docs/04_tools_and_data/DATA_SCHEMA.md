# Data Schemas

This document provides formal schemas for the data structures used in CueDeck. Use these for validation and implementation.

## 1. Card Frontmatter (YAML)

**File Path**: `.cuedeck/cards/*.md`

```yaml
type: object
required: [id, title]
properties:
  id:
    type: string
    pattern: "^[a-z0-9]{6}$"
    description: "6-character alphanumeric hash (stateless ID)"
  uuid:
    type: string
    format: "uuid"
    description: "System-generated V4 UUID for audit trails"
  title:
    type: string
    description: "Human-readable title of the task"
  status:
    type: string
    enum: ["todo", "active", "done", "archived"]
    default: "todo"
  priority:
    type: string
    enum: ["low", "medium", "high", "critical"]
    default: "medium"
  assignee:
    type: string
    nullable: true
  tags:
    type: array
    items: { type: string }
  refs:
    type: array
    items:
      type: string
      description: "Relative path or anchor (e.g., 'docs/api.md#Login')"
  created_at:
    type: string
    format: "date-time"
  last_modified:
    type: string
    format: "date-time"
```

## 2. Metadata Cache (JSON)

**File Path**: `.cuedeck/.cache/metadata.json`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CueDeck Metadata Cache",
  "type": "object",
  "required": ["version", "files", "created_at", "last_updated"],
  "properties": {
    "version": {
      "type": "string",
      "const": "2.1"
    },
    "created_at": {
      "type": "integer",
      "description": "Unix timestamp when cache was created"
    },
    "last_updated": {
      "type": "integer",
      "description": "Unix timestamp of last cache update"
    },
    "files": {
      "type": "object",
      "additionalProperties": {
        "$ref": "#/definitions/FileEntry"
      }
    }
  },
  "definitions": {
    "FileEntry": {
      "type": "object",
      "required": ["hash", "last_checked_ts", "token_count"],
      "properties": {
        "hash": {
          "type": "string",
          "pattern": "^[a-f0-9]{64}$",
          "description": "SHA-256 hash of file content"
        },
        "last_checked_ts": { "type": "integer" },
        "token_count": { "type": "integer" },
        "anchors": {
          "type": "array",
          "items": { "type": "string" }
        },
        "dependencies": {
          "type": "array",
          "items": { "type": "string" },
          "description": "List of paths this file references"
        }
      }
    }
  }
}
```

## 3. Session State (JSON)

**File Path**: `.cuedeck/.cache/sessions/*.json`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "session_id": { "type": "string", "format": "uuid" },
    "start_time": { "type": "string", "format": "date-time" },
    "workflow": { "type": "string" },
    "working_set": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "path": { "type": "string" },
          "hash": { "type": "string" },
          "role": { "enum": ["read", "write"] }
        }
      }
    },
    "decisions": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "decision": { "type": "string" },
          "rationale": { "type": "string" },
          "timestamp": { "type": "string", "format": "date-time" }
        }
      }
    },
    "tokens_used": { "type": "integer" }
  }
}
```

## 4. MCP Configuration (JSON Fragment)

**Context**: Inside `claude_desktop_config.json` or VSCode settings.

```json
{
  "mcpServers": {
    "cuedeck": {
      "command": "cue",
      "args": ["mcp"],
      "env": {
        "RUST_LOG": "info",
        "CUEDECK_PATH": "/abs/path/to/project"
      },
      "disabled": false,
      "autoApprove": []
    }
  }
}
```

---
**Related Docs**: [KNOWLEDGE_BASE_STRUCTURE.md](../04_tools_and_data/KNOWLEDGE_BASE_STRUCTURE.md), [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md)
