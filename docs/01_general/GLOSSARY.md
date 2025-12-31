# Glossary

## Core Concepts

| Term | Definition |
| :--- | :--- |
| **Workspace** | The root directory containing `.cuedeck/` and all managed files. |
| **Card** | A `.md` file in `.cuedeck/cards/` representing a single unit of work (Task, Bug, Feature). |
| **Task** | The *logical* work item described *within* a Card. A Card is the file; a Task is the content. |
| **Doc** | A `.md` file in `.cuedeck/docs/` representing long-lived knowledge (API, Architecture). |
| **Scene** | The compiled, pruned, and secure context output file (`.cuedeck/SCENE.md`). |
| **Anchor** | A named location within a file, identified by a Markdown Header (e.g., `#Login Flow`). |
| **Ref / Reference** | An explicit link from one file to another (or to an Anchor), defined in `refs:` frontmatter. |

## Data Structures

| Term | Definition |
| :--- | :--- |
| **Frontmatter** | YAML metadata block at the top of a Markdown file, enclosed by `---`. |
| **Metadata Cache** | The JSON file (`.cuedeck/.cache/metadata.json`) storing hashes and token counts. |
| **DAG** | Directed Acyclic Graph. The internal representation of file references used to prevent cycles. |
| **Hash ID** | 6-character alphanumeric identifier generated from timestamp + title to prevent Git conflicts. |
| **Working Set** | Files currently in-focus based on active cards and their transitive dependencies. |

## Processes

| Term | Definition |
| :--- | :--- |
| **Hot Path** | The optimized code execution path for incremental updates (target: <5ms). |
| **Cold Start** | The initial full-parse when no cache exists (target: <1s for 100 files). |
| **Lazy GC** | Garbage Collection triggered on-access (not on a schedule). Removes stale cache entries. |
| **Token Pruning** | The process of trimming context to fit within an LLM's token budget. |
| **Secret Masking** | Regex-based replacement of sensitive strings (API keys) with `***`. |
| **Anchor Walking** | Graph traversal strategy that follows `refs:` links to build context. |
| **Scene Generation** | Process of compiling active cards + refs into a single contextual SCENE.md. |

## Protocols

| Term | Definition |
| :--- | :--- |
| **MCP** | Model Context Protocol. The JSON-RPC 2.0 standard for AI tool communication. |
| **Stdio Isolation** | The rule that `stdout` is ONLY for RPC responses; logs go to `stderr`. |
| **JSON-RPC 2.0** | The remote procedure call protocol used for MCP communication. |

## Technical Terms

| Term | Definition |
| :--- | :--- |
| **Config Cascading** | Override mechanism: Global Config → Project Config → CLI Flags (higher priority wins). |
| **Pruning** | The process of cutting context content to fit within the LLM token budget. |
| **Debounce** | Technique to aggregate rapid events (e.g., 500ms delay) before triggering an action. |
| **Zombie Entry** | A cache entry for a file that no longer exists on disk (removed on next GC). |
| **Cache Rot** | Situation where cache metadata becomes out of sync with actual file state. |
| **Token Budget** | Maximum number of tokens allowed in generated context (default: 32000). |
| **Priority Score** | Numeric value determining which files to include when pruning (higher = more important). |

## Rust/Crate Terms

| Term | Definition |
| :--- | :--- |
| **Crate** | A Rust package (library or binary). CueDeck has 5 crates: `cue_common`, `cue_config`, `cue_core`, `cue_cli`, `cue_mcp`. |
| **Workspace** (Cargo) | Rust's multi-crate project structure, defined in root `Cargo.toml`. |
| **miette** | Rust crate for pretty error reporting with `help` and `code` annotations. |
| **thiserror** | Rust crate for deriving custom `Error` enums with `#[error(...)]` attributes. |
| **tokio** | Async runtime providing async I/O and task scheduling. |
| **serde** | Rust (de)serialization framework for JSON, TOML, YAML. |
| **clap** | CLI argument parser with derive-macro support. |
| **skim** | Fuzzy finder TUI library (Rust port of `fzf`). |
| **notify** | Cross-platform file watcher crate. |

## CLI Commands Reference

| Command | Description |
| :--- | :--- |
| `cue init` | Initialize `.cuedeck/` directory structure in current folder. |
| `cue scene` | Generate and copy `SCENE.md` context to clipboard. |
| `cue open` | Launch fuzzy finder TUI to open a card or doc. |
| `cue watch` | Background watcher that auto-updates clipboard on file changes. |
| `cue doctor` | Health check for workspace config, dead links, and cache. |
| `cue card` | Manage task cards (new, list, archive). |
| `cue clean` | Remove cache and force cold-start rebuild. |
| `cue upgrade` | Self-update to latest CueDeck version. |
| `cue mcp` | Start MCP server for AI tool integration. |

## MCP Error Codes

| Code | Name | Description |
| :--- | :--- | :--- |
| `1001` | `FileNotFound` | Referenced file does not exist. |
| `1002` | `CyclicDependency` | Circular reference detected in DAG. |
| `1003` | `TokenLimitExceeded` | Context exceeds configured token budget. |
| `1004` | `InvalidFrontmatter` | YAML parsing error in file header. |
| `-32600` | `InvalidRequest` | Malformed JSON-RPC request. |
| `-32601` | `MethodNotFound` | Unknown MCP method called. |
| `-32602` | `InvalidParams` | Invalid parameters for method. |
| `-32603` | `InternalError` | Unexpected server-side error. |

---
**Related Docs**: [PROJECT_OVERVIEW.md](./PROJECT_OVERVIEW.md), [SYSTEM_ARCHITECTURE.md](../02_architecture/SYSTEM_ARCHITECTURE.md), [MEMORY_STRATEGY.md](../03_agent_design/MEMORY_STRATEGY.md), [CLI_REFERENCE.md](../04_tools_and_data/CLI_REFERENCE.md)
