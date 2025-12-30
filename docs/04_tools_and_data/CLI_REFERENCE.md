# CLI Reference

## Global Flags

- `--help`: Print help information.
- `--version`: Print version information.
- `--verbose`: Enable DEBUG level logging (prints to stderr).

## Commands

### `cue init`

Initializes the `.cuedeck` structure in the current directory.

- **Behavior**: Safe idempotent (won't overwrite existing config unless forced).
- **Files Created**:
  - `.cuedeck/config.toml`
  - `.cuedeck/cards/` (Empty)
  - `.cuedeck/docs/` (Empty)
  - `.gitignore` (Appends `.cuedeck/.cache`)

### `cue scene`

Generates the context for the LLM.

- **Flags**:
  - `-d`, `--dry-run`: Output to stdout instead of System Clipboard.
  - `--token-limit <N>`: Override config token limit for this run.
- **Output**:
  - `stderr`: "Scene built in 4ms. 12,400 tokens."
  - `stdout`/`clipboard`: The actual Markdown content.

### `cue open [QUERY]`

Launches the interactive TUI (Skim).

- **Arguments**:
  - `QUERY`: Optional initial search filter.
- **Interaction**:
  - `Up/Down`: Navigate results.
  - `Enter`: Open selected file in `$EDITOR`.
  - `Esc`: Exit.

### `cue watch`

Starts the monitoring daemon.

- **Behavior**: Blocks the terminal. Runs forever.
- **Process**:
  - On file change -> Re-runs `cue scene` logic (in-memory).
  - Updates Clipboard.
  - Prints timestamp to stderr.

### `cue doctor`

Diagnoses workspace issues.

- **Checks**:
  - [x] Config Syntax
  - [x] Path Validity
  - [x] Circular References
  - [x] Orphan Tasks
- **Exit Code**:
  - `0`: All clear.
  - `1`: Issues found.

### `cue card`

Manage implementation tasks.

- **Subcommands**:
  - `new <TITLE>`: Creates a new card with a unique ID (e.g., `cue card new "Fix Login"` -> `cards/2a9f1x.md`).

### `cue clean`

Hard reset of the cache.

- **Action**: `rm -rf .cuedeck/.cache`
- **Use Case**: recovering from "Cache Rot" or corrupted metadata.

---
**Related Docs**: [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md), [USER_STORIES.md](../01_general/USER_STORIES.md)
