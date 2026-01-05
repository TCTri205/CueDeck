# Agent README

> **For AI Agents**: This document explains how to interact with this CueDeck workspace.

## Quick Start

```bash
# Search for context
cue search "authentication" --json

# List active tasks  
cue list --json

# Create a task
cue card create "Implement feature X" --tags backend,api --priority high

# Validate workspace
cue doctor --strict --json
```

## Workspace Structure

```
.cuedeck/
├── config.toml      # Configuration
├── cards/           # Task cards (*.md files)
├── docs/            # Documentation files
├── cache/           # Cache files (gitignored)
└── hooks/           # Git hooks (pre-commit)
```

## MCP Tools Available

| Tool | Purpose | Example |
|:-----|:--------|:--------|
| `read_context` | Search documents | `{"query": "auth", "mode": "hybrid"}` |
| `read_doc` | Read specific file | `{"path": "docs/api.md", "anchor": "Login"}` |
| `list_tasks` | List task cards | `{"status": "active", "priority": "high"}` |
| `create_task` | Create new task | `{"title": "Fix bug", "priority": "high"}` |
| `update_task` | Modify task | `{"id": "abc123", "updates": {"status": "done"}}` |
| `get_task_dependencies` | Get task deps | `{"id": "abc123", "reverse": false}` |
| `validate_task_graph` | Check for cycles | `{}` |

## Task Card Schema

```yaml
---
title: "Verb + Component + Outcome"  # Required
status: todo | active | done | archived
priority: low | medium | high | critical
assignee: "@username"
tags: [backend, auth]
depends_on: [abc123, def456]  # Must exist!
created: "2026-01-04T12:00:00Z"
updated: "2026-01-04T12:00:00Z"  # Auto-updated
---

## Description
[Task content here]
```

## Search Modes

- **`keyword`**: Fast exact matching (~50ms)
- **`semantic`**: AI similarity search (~200ms cached)
- **`hybrid`** (default): 70% semantic + 30% keyword

## Filters

```bash
# Filter by tags (OR logic)
cue search "api" --tags auth,security

# Filter by priority
cue list --priority high

# Filter by assignee
cue list --assignee @developer

# Filter by date
cue list --created ">7d"  # Last 7 days
```

## Validation Rules

Before creating/updating tasks, these rules are enforced:

1. **Title**: Required, non-empty, max 200 chars
2. **Priority**: Must be low/medium/high/critical
3. **depends_on**: All IDs must exist (checked)
4. **No cycles**: Circular dependencies blocked

## Error Handling

| Error | Code | Recovery |
|:------|:-----|:---------|
| File Not Found | 1001 | Check path, use fuzzy search |
| Cycle Detected | 1002 | Remove conflicting dependency |
| Token Limit | 1003 | Reduce scope or use anchors |
| Dependency Not Found | - | Verify task ID exists |
| Validation Error | - | Fix input per schema |

## Best Practices

1. **Always validate**: Run `cue doctor` after modifications
2. **Use dependencies**: Connect related tasks with `depends_on`
3. **Be specific**: Use anchors to read specific sections
4. **Check before create**: Verify dependency IDs exist first

---
**Full Docs**: See `docs/00_INDEX.md` for complete documentation.
