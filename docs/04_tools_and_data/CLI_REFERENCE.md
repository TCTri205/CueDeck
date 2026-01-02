# CLI Reference

## Global Flags

- `--help`: Print help information.
- `--version`: Print version information.
- `--verbose`: Enable DEBUG level logging (prints to stderr).

## Exit Codes

| Code | Meaning | Action |
| :--- | :--- | :--- |
| `0` | **Success** | Proceed normally. |
| `1` | **General Error** | Check stderr for details. |
| `2` | **Usage Error** | Invalid flags/arguments. Check `--help`. |
| `101` | **Config Error** | Invalid `config.toml`. Run `cue doctor`. |
| `130` | **Terminated** | User pressed Ctrl+C. |

## Commands

### `cue init`

Initializes the `.cuedeck` structure in the current directory.

- **Behavior**: Safe idempotent (won't overwrite existing config unless forced).
- **Files Created**:
  - `.cuedeck/` — Workspace root directory
  - `.cuedeck/config.toml` — Default configuration (auto-fills `author` from Global Config)
  - `.cuedeck/cards/` — Empty directory for task cards
  - `.cuedeck/docs/` — Empty directory for documentation
  - `.gitignore` — Appends patterns: `.cuedeck/.cache`, `.cuedeck/SCENE.md`

### `cue scene`

Generates the context for the LLM.

- **Flags**:
  - `-d`, `--dry-run`: Output to stdout instead of System Clipboard.
  - `--token-limit <N>`, `--limit <N>`: Override config token limit for this run.
- **Output**:
  - `stderr`: "Scene built in 4ms. 12,400 tokens."
  - `stdout`/`clipboard`: The actual Markdown content.

### `cue open [QUERY]`

Launches the interactive file selector with configurable search mode.

- **Arguments**:
  - `QUERY`: Optional initial search filter.
    - **Tip**: To start a search with filters but no query, simple omit the query: `cue open --tags auth` or use empty quotes `cue open "" --tags auth`.

- **Flags**:
  - `--mode=<MODE>`: Search mode selection (default: `hybrid`)
    - `keyword`: Fast exact/fuzzy text matching (~50ms)
    - `semantic`: AI-powered conceptual search (~2-5s first run, ~200ms cached)
    - `hybrid`: **Default** - Combines both with 70/30 weighting (~250ms cached)
  - `--tags <TAGS>`: Filter by tags (comma-separated, e.g., "auth,api"). ANY match logic.
  - `--priority <PRIORITY>`: Filter by priority (e.g., "high", "medium", "low"). Case-insensitive.
  - `--assignee <ASSIGNEE>`: Filter by assignee (e.g., "@tctri"). Case-insensitive.
  - `--semantic`: **Deprecated** - Use `--mode=semantic` instead (kept for backward compatibility)

- **Search Behavior**:

  | Mode | Filename Weight | Content Weight | Embedding | Cache |
  | :--- | :--- | :--- | :--- | :--- |
  | `keyword` | 100 (exact match) | 10 (token match) | No | No |
  | `semantic` | 0 | Cosine similarity | Yes | Yes |
  | `hybrid` | 30% of total | 70% of total | Yes | Yes |

- **Interaction**:
  - `Type`: Filter results
  - `Up/Down`: Navigate results
  - `Enter`: Open selected file in `$EDITOR`
  - `Esc`: Exit

- **Examples**:

  ```bash
  # Fast keyword search (exact matches)
  cue open "authentication"
  
  # Conceptual search (understands synonyms)
  cue open "concurrent programming" --mode=semantic
  
  # Hybrid (default, best relevance)
  cue open "login flow"
  cue open "error handling" --mode=hybrid  # Explicit
  
  # Filter by tags
  cue open "authentication" --tags auth,security

  # Filter by priority
  cue open database --priority high

  # Combined filters
  cue open api --tags backend --priority high --assignee @dev
  
  # Legacy syntax (still supported)
  cue open "auth" --semantic
  ```

- **Performance**:
  - First semantic/hybrid search: ~2-5s (downloads 22MB model + generates embeddings)
  - Subsequent searches: ~200-300ms (uses cached embeddings)
  - Cache stored in: `.cuedeck/cache/embeddings.bin`

### `cue watch`

Starts the monitoring daemon.

- **Behavior**: Blocks the terminal. Runs forever.
- **Process**:
  - Monitors `.cuedeck/` and `src/` (recursively) for file events.
  - **Debounce**: Waits 500ms after the last event.
  - **Action**:
    - Re-runs `cue scene` logic (in-memory).
    - Updates System Clipboard with the new context.
    - Prints timestamp and new token count to stderr.
- **Exclusions**: Ignores `.git`, `target`, `.cache`, and writes to `SCENE.md`.

### `cue doctor`

Diagnoses workspace issues using the `miette` library for beautiful error reporting.

- **Checks**:
  - [x] Config Syntax — Valid TOML structure
  - [x] YAML Frontmatter — Validates frontmatter in all Cards
  - [x] Path Validity — Checks all reference paths exist
  - [x] Dead Links — Detects references to non-existent files or anchors
  - [x] Circular Dependencies — Uses graph algorithm (DFS/Tarjan's) to detect cycles
  - [x] Orphan Tasks — Warns about active cards with no assignee
- **Flags**:
  - `--repair`: Attempt automatic fixes for detected issues.
  - `--json`: Output results as JSON (machine-readable).
  - `--format=<text|json>`: Specify output format (default: text).
- **Exit Code**:
  - `0`: All clear.
  - `1`: Issues found.

### `cue graph`

Visualize and analyze dependency graph of markdown documents.

- **Usage**:

  ```bash
  cue graph [OPTIONS]
  ```

- **Flags**:
  - `--format <FORMAT>`: Output format (`ascii`, `mermaid`, `dot`, `json`). Default: `ascii`.
  - `--output <FILE>`: Write output to file instead of stdout.
  - `--stats`: Show graph statistics (nodes, edges, cycles, orphans).

- **Output Formats**:
  - **`ascii`**: Terminal-friendly text representation
  - **`mermaid`**: Mermaid flowchart syntax (for GitHub/docs)
  - **`dot`**: Graphviz DOT format
  - **`json`**: Machine-readable JSON structure

- **Examples**:

  ```bash
  # Show ASCII visualization
  cue graph
  
  # Export Mermaid diagram
  cue graph --format mermaid --output docs/graph.md
  
  # Show statistics with visualization
  cue graph --format ascii --stats
  
  # Export JSON for external tools
  cue graph --format json --output graph.json
  ```

- **Graph Statistics** (with `--stats`):
  - **Nodes**: Total documents in workspace
  - **Edges**: Dependencies between documents
  - **Cycles**: Whether circular dependencies exist
  - **Orphans**: Documents with no incoming links

### `cue card`

Manage implementation tasks.

- **Subcommands**:
  - `new <TITLE>`: Creates a new card with a unique ID (e.g., `cue card new "Fix Login"` -> `cards/2a9f1x.md`).
  - `list [--status=<STATUS>]`: List all cards, optionally filtered by status (`active`, `archived`, `all`).
  - `edit <ID>`: Open card in `$EDITOR`.
  - `archive <ID>`: Move card to archived status.

### `cue list`

Alias for `cue card list`. Lists all cards.

- **Flags**:
  - `--status=<active|archived|all>`: Filter by status (default: active).

### `cue clean`

Hard reset of the cache.

- **Flags**:
  - `--logs`: Also clear log files in `.cuedeck/logs/`.
- **Action**: `rm -rf .cuedeck/.cache`
- **Use Case**: Recovering from "Cache Rot" or corrupted metadata.

### `cue logs`

Manage log files.

- **Subcommands**:
  - `archive`: Rotate and compress old log files to `.cuedeck/logs/archive/`.
  - `clear`: Remove all log files.

### `cue upgrade`

Self-updates CueDeck to the latest version (Module 4).

- **Behavior**:
  - Checks GitHub Releases API for latest version.
  - Compares semantic versions (e.g., v2.1.0 vs v2.2.0).
  - Downloads and replaces the binary if a new release exists.
  - Handles Private Repos/404s gracefully by warning the user.
- **Output**:
  - `stderr`: "Checking for updates..."
  - `stderr`: "! New version available: 2.2.0 (Current: 2.1.0)"
  - `stderr`: "Release URL: <https://github.com/>..."
- **Exit Code**:
  - `0`: Success (No update needed or Upgrade complete).
  - `1`: Network or IO error.

### `cue mcp`

Starts the MCP (Model Context Protocol) Server for AI integration (Module 3).

- **Behavior**:
  - Spawns JSON-RPC 2.0 server over `stdio`.
  - **CRITICAL**: `stdout` is reserved EXCLUSIVELY for JSON-RPC responses.
  - All logs (Info/Warn/Error) go to `stderr` to avoid protocol corruption.
- **Configuration** (in AI tool config, e.g., Claude Desktop):

  ```json
  {
    "mcpServers": {
      "cuedeck": {
        "command": "cue",
        "args": ["mcp"]
      }
    }
  }
  ```

- **Supported Methods**:
  - `read_context(query, limit, mode, semantic)` — Fuzzy, semantic, or hybrid search across context
    - `mode` (optional, string): Search mode: `keyword`, `semantic`, or `hybrid` (default: `hybrid`)
    - `semantic` (optional, boolean): **Deprecated** - Use `mode` parameter instead (backward compatibility)
  - `read_doc(path, anchor)` — Read specific document or section
  - `list_tasks(status)` — List cards by status
  - `update_task(id, updates)` — Modify card frontmatter

---
**Related Docs**: [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md), [USER_STORIES.md](../01_general/USER_STORIES.md), [TOOLS_SPEC.md](./TOOLS_SPEC.md)

## `cue card` commands

Manage task cards with rich metadata and dependency tracking.

### `cue card create`

Create a new task card with optional metadata.

- **Usage**:

  ```bash
  cue card create <title> [OPTIONS]
  ```

- **Arguments**:
  - `<title>`: Task title (required)

- **Flags**:
  - `--tags, -t <tags>`: Comma-separated tags for categorization
  - `--priority, -p <priority>`: Task priority (`low`, `medium`, `high`, `critical`, default: `medium`)
  - `--assignee, -a <assignee>`: Person assigned to the task
  - `--depends-on, -d <task-ids>`: Comma-separated task IDs this task depends on

- **Examples**:

  ```bash
  # Simple task
  cue card create "Implement login feature"
  
  # Task with metadata
  cue card create "Add user authentication" \
    --tags auth,backend \
    --priority high \
    --assignee @developer
  
  # Task with dependencies
  cue card create "Add login UI" \
    --depends-on abc123,def456 \
    --tags frontend,auth
  ```

- **Output**: Creates `.cuedeck/cards/<ID>.md` and displays task ID
- **Validation**:
  - Checks that dependency task IDs exist
  - Prevents circular dependencies

### `cue card deps`

Show task dependencies or dependents.

- **Usage**:

  ```bash
  cue card deps <task-id> [--reverse]
  ```

- **Arguments**:
  - `<task-id>`: 6-character task ID

- **Flags**:
  - `--reverse, -r`: Show tasks that depend on this task (reverse dependencies)

- **Examples**:

  ```bash
  # Show what task xyz789 depends on
  cue card deps xyz789
  Output:
    Dependencies for xyz789 (Implement login):
      → abc123: Setup auth framework
      → def456: Create user database
  
  # Show what depends on xyz789
  cue card deps xyz789 --reverse
  Output:
    Tasks depending on xyz789 (Implement login):
      ← ghi012: Add login UI
      ← jkl345: Add password reset
  ```

### `cue card validate`

Validate task dependency graph for circular dependencies.

- **Usage**:

  ```bash
  cue card validate [<task-id>]
  ```

- **Arguments**:
  - `<task-id>` (optional): Validate specific task only

- **Examples**:

  ```bash
  # Validate entire task graph
  cue card validate
  Output:
    ✅ All task dependencies are valid (no circular dependencies)
  
  # Validate specific task
  cue card validate xyz789
  Output:
    ✅ Task xyz789 dependencies are valid
  
  # Example error (circular dependency detected)
  cue card validate
  Output:
    ❌ Circular dependency detected: abc123 → def456 → ghi789 → abc123
  ```

- **Exit Code**:
  - `0`: All dependencies valid
  - `1`: Circular dependency detected

---
