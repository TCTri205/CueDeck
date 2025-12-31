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

Launches the interactive TUI (Skim).

- **Arguments**:
- **Arguments**:
  - `QUERY`: Optional initial search filter.
- **Interaction**:
  - `Type`: Filter files by filename (high weight) or content (token match).
  - `Up/Down`: Navigate results.
  - `Enter`: Open selected file in `$EDITOR`.
  - `Esc`: Exit.

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
  - `read_context(query, limit)` — Fuzzy search across context
  - `read_doc(path, anchor)` — Read specific document or section
  - `list_tasks(status)` — List cards by status
  - `update_task(id, updates)` — Modify card frontmatter

---
**Related Docs**: [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md), [USER_STORIES.md](../01_general/USER_STORIES.md), [TOOLS_SPEC.md](./TOOLS_SPEC.md)
