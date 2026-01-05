# Query Language Reference

**Version**: 2.0  
**Status**: Production Ready  
**Introduced**: Week 2 - Agent API Enhancement

---

## Overview

The unified query language provides a simple, consistent syntax for filtering tasks across both MCP and CLI interfaces. It replaces multiple filter parameters with a single query string.

**Format**: `field:value [field:value...] [+tag] [-tag]`

---

## Syntax

### Field Filters

```
field:value         # Exact match
field:>value        # Greater than (dates)
field:<value        # Less than (dates)
```

### Tag Filters

```
+tag                # Must have tag
-tag                # Must NOT have tag
```

### Combining Filters

All filters use **AND logic** - all conditions must match.

```
status:active priority:high +backend -archived
```

Means: status is "active" AND priority is "high" AND has "backend" tag AND does NOT have "archived" tag

---

## Supported Fields

| Field | Values | Examples |
|:------|:-------|:---------|
| `status` | todo, active, done, archived | `status:active` |
| `priority` | low, medium, high, critical | `priority:high` |
| `assignee` | @username | `assignee:@dev` |
| `created` | Date or relative | `created:>7d`, `created:<2024-01-01` |
| `updated` | Date or relative | `updated:>2w` |

---

## Date Formats

### Absolute Dates

```
created:2024-01-01        # Exact date (YYYY-MM-DD)
created:2024-01           # Month (YYYY-MM)
created:2024              # Year (YYYY)
```

### Relative Dates

```
created:>7d               # Created within last 7 days
updated:<30d              # Not updated in 30 days
```

**Supported units**:

- `d` - days (e.g., `7d`)
- Week/month/year may be added in future

---

## Examples

### Simple Queries

```
# All active tasks
status:active

# High priority tasks
priority:high

# Tasks assigned to developer
assignee:@dev
```

### Tag Queries

```
# Backend tasks
+backend

# Not archived
-archived

# Backend but not archived
+backend -archived
```

### Combined Queries

```
# Active backend tasks
status:active +backend

# High priority tasks created recently
priority:high created:>7d

# Active backend tasks not assigned
status:active +backend -assigned

# Complex query
status:active priority:high +backend -archived created:>7d
```

---

## Usage

### CLI

```bash
# Single query
cue query "status:active +backend" --json

# Batch from file
cue query --batch queries.json --json

# Human-readable output
cue query "status:active priority:high"
```

### MCP Tool: `batch_query`

```json
{
  "name": "batch_query",
  "arguments": {
    "queries": [
      {
        "id": "q1",
        "query_string": "status:active +backend",
        "limit": 10
      }
    ]
  }
}
```

---

## Performance

- **Single query**: ~50ms (typical workspace)
- **Batch 50 queries**: <100ms (shared workspace scan)
- **Optimization**: Use batch mode for multiple queries

---

## Error Handling

### Invalid Field

```
invalid_field:value
```

**Error**: `Unknown field 'invalid_field'. Valid fields: status, priority, assignee, created, updated`

### Invalid Syntax

```
status active  # Missing colon
```

**Error**: `Invalid token 'status'. Expected 'field:value', '+tag', or '-tag'`

### Empty Values

```
status:       # Empty value
+             # Empty tag
```

**Error**: `Empty value for field 'status'` or `Empty tag after '+'`

---

## Comparison to Old API

### Before (Multiple Parameters)

```json
{
  "status": "active",
  "tags": ["backend"],
  "priority": "high",
  "created": ">7d"
}
```

### After (Unified Query)

```json
{
  "query_string": "status:active priority:high +backend created:>7d"
}
```

**Benefits**:

- ✅ Simpler API (1 parameter vs 4+)
- ✅ More powerful (tag exclusion with `-`)
- ✅ Consistent across MCP/CLI
- ✅ Human-readable

---

## Future Enhancements

Planned for future versions:

1. **OR logic**: `(status:active | status:todo)`
2. **Regex**: `title:~"Implement.*"`  
3. **Wildcards**: `assignee:@team-*`
4. **More date units**: `2w`, `3m`, `1y`
5. **Saved queries**: `query:my-saved-query`

---

## See Also

- [CLI Reference](./CLI_REFERENCE.md#query-command)
- [MCP Tools Spec](./TOOLS_SPEC.md#batch_query)
- [Task Filters](./DATA_SCHEMA.md#task-filters)
