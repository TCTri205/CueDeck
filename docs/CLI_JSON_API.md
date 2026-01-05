# Using CueDeck CLI with JSON Output

## Overview

CueDeck CLI now supports comprehensive JSON output for seamless integration with tools and extensions. All JSON-enabled commands use the `--json` flag.

## Commands with JSON Support

### 1. Search Command

Search documents with real relevance scoring (0.0-1.0 range).

**Syntax:**

```bash
cue search <query> --json [--mode <mode>] [--limit <n>]
```

**Search Modes:**

- `keyword` - Fast TF-IDF based search
- `semantic` - AI-powered semantic search  
- `hybrid` - Combined keyword + semantic (default)

**Example:**

```bash
cue search "authentication" --mode hybrid --json
```

**JSON Output:**

```json
[
  {
    "path": "D:/workspace/docs/auth.md",
    "score": "0.92",
    "preview": "Authentication System"
  },
  {
    "path": "D:/workspace/tasks/login.md",
    "score": "0.67",
    "preview": "Implement OAuth2 Login"
  }
]
```

**Fields:**

- `path` - Absolute path to the document
- `score` - Relevance score as string (formatted to 2 decimals)
- `preview` - Document title or filename

---

### 2. List Command

List tasks with complete metadata and filtering options.

**Syntax:**

```bash
cue list --status <status> --json [--priority <p>] [--assignee <a>]
```

**Example:**

```bash
cue list --status active --priority high --json
```

**JSON Output:**

```json
[
  {
    "id": "abc123",
    "title": "Implement User Authentication",
    "status": "active",
    "priority": "high",
    "assignee": "alice",
    "tags": ["backend", "security"],
    "file": "D:/workspace/tasks/abc123.md",
    "line": 1,
    "created": "2026-01-01",
    "updated": "2026-01-03",
    "dependsOn": ["xyz789"]
  }
]
```

**Fields:**

- `id` - Unique task identifier (6 chars)
- `title` - Task title from frontmatter
- `status` - Task status (`todo`, `active`, `done`, `archived`)
- `priority` - Priority level (`low`, `medium`, `high`)
- `assignee` - Assigned person (optional)
- `tags` - Array of tags (optional)
- `file` - Absolute path to task file
- `line` - Line number (always 1 for frontmatter)
- `created` - Creation timestamp (optional)
- `updated` - Last update timestamp (optional)
- `dependsOn` - Array of dependency task IDs (optional)

---

### 3. Graph Command

Export complete dependency graph topology.

**Syntax:**

```bash
cue graph --json [-o <output-file>]
```

**Example:**

```bash
cue graph --json -o graph.json
```

**JSON Output:**

```json
{
  "nodes": [
    {
      "id": "auth.md",
      "path": "D:/workspace/tasks/auth.md",
      "type": "task",
      "title": "Authentication System",
      "metadata": {
        "status": "active",
        "priority": "high"
      }
    }
  ],
  "edges": [
    {
      "from": "auth.md",
      "to": "database.md",
      "type": "dependency"
    }
  ],
  "stats": {
    "node_count": 42,
    "edge_count": 67,
    "has_cycles": false
  }
}
```

**Structure:**

- `nodes` - Array of graph nodes
  - `id` - Node identifier (filename)
  - `path` - Absolute file path
  - `type` - Node type (`task` or `document`)
  - `title` - Document title (optional)
  - `metadata` - Task metadata (optional)
- `edges` - Array of graph edges  
  - `from` - Source node ID
  - `to` - Target node ID
  - `type` - Edge type (always `dependency`)
- `stats` - Graph statistics
  - `node_count` - Total number of nodes
  - `edge_count` - Total number of edges
  - `has_cycles` - Whether graph contains cycles

---

## Error Handling

All commands return structured error responses in JSON mode:

**Error Response Format:**

```json
{
  "success": false,
  "error": {
    "code": "SEARCH_FAILED",
    "message": "Failed to execute search: index not found"
  }
}
```

**Error Codes:**

- `CWD_ERROR` - Failed to get current working directory
- `SEARCH_FAILED` - Search operation failed
- `LIST_FAILED` - List operation failed
- `INVALID_DATE_FILTER` - Invalid date filter format
- `GRAPH_BUILD_FAILED` - Graph construction failed

---

## Integration Example (TypeScript)

```typescript
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

// Search documents
async function searchDocs(query: string) {
  const { stdout } = await execAsync(`cue search "${query}" --json`);
  return JSON.parse(stdout);
}

// List active tasks
async function listActiveTasks() {
  const { stdout } = await execAsync('cue list --status active --json');
  return JSON.parse(stdout);
}

// Export graph
async function exportGraph() {
  const { stdout } = await execAsync('cue graph --json');
  return JSON.parse(stdout);
}
```

---

## Backward Compatibility

All commands maintain full backward compatibility. When `--json` flag is NOT used, commands display human-readable output as before.

**Example (Human-readable):**

```bash
$ cue search "auth"
Found 3 results:
1. [0.92] D:/workspace/docs/auth.md
2. [0.67] D:/workspace/tasks/login.md  
3. [0.54] D:/workspace/tasks/oauth.md
```

---

## Performance Notes

- **Search**: Semantic search may take 1-2 seconds on first run (model loading). Subsequent searches are cached.
- **List**: Fast, typically < 100ms for workspaces with < 1000 tasks.
- **Graph**: Construction time depends on workspace size. Typical: 100-500ms for 100-1000 files.

---

## VSCode Extension

The CueDeck VSCode extension uses these JSON APIs. See `extensions/vscode/` for implementation details.
