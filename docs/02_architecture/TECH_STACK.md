# Technology Stack

## Core Language

- **Rust**: Selected for performance, safety, and workspace capabilities.
- **Minimum Version**: `1.75.0` (Required for `async fn` in traits, stabilized in 1.75)
- **Recommended**: Latest **stable** channel
- **MSRV Policy**: Update MSRV only for critical language features, not patch releases

## Key Crates (Libraries)

### Core Dependencies (`Cargo.toml`)

```toml
[workspace.dependencies]
# CLI & Args
clap = { version = "4.5", features = ["derive", "string"] }
# Async Runtime
tokio = { version = "1.36", features = ["full", "tracing"] }
# Error Handling
miette = { version = "7.2", features = ["fancy"] }
thiserror = "1.0.57"
# Serialization
serde = { version = "1.0.197", features = ["derive"] }
serde_json = "1.0.114"
serde_yaml = "0.9.33"
# Parsing
pulldown-cmark = "0.10"
gray_matter = "0.2"
tiktoken-rs = "0.5"
# File System
walkdir = "2.5"
globset = "0.4"
notify = "6.1"
# Hashing
sha2 = "0.10"
# Config
config = "0.14"
# UI
skim = "0.10"
# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Dependency Rationale

#### CLI & Argument Parsing

- **`clap`** (v4.5+): Industry-standard CLI library with derive macros for type-safe args.
  - **Why Not**: `structopt` (deprecated, merged into clap).
  - **Features**: `derive` for ergonomics, `string` for better UTF-8 handling.

#### Async Runtime

- **`tokio`** (v1.36+): De facto async runtime for Rust.
  - **Why Not**: `async-std` (smaller ecosystem), `smol` (less mature for prod).
  - **Features**: `full` feature set required for multi-threaded scheduler (MCP server needs concurrency).
  - **Tracing Integration**: `tracing` feature for observability.

#### Error Handling

- **`miette`** (v7.2+): Beautiful diagnostic errors with source code spans.
  - **Why Not**: `anyhow` (less user-friendly output for CLI tools).
  - **Features**: `fancy` for colored terminal output with code snippets.
  - **Use Case**: User-facing tool requires clear error messages.

- **`thiserror`** (v1.0.57+): Ergonomic custom error types with derive macros.
  - **Complements**: `miette` (thiserror defines errors, miette displays them).

#### Serialization

- **`serde`** (v1.0.197+): Universal serialization framework.
  - **Features**: `derive` for automatic trait implementation.
  - **Formats**: `serde_json` (MCP protocol), `serde_yaml` (frontmatter), `toml` (config).

#### Markdown Parsing

- **`pulldown-cmark`** (v0.10): CommonMark-compliant parser.
  - **Why Not**: `comrak` (heavier), `markdown-rs` (less mature).
  - **Performance**: Streaming parser, zero-copy where possible.

- **`gray_matter`** (v0.2): YAML frontmatter extraction.
  - **Limitation**: Only supports YAML, not TOML frontmatter (acceptable for CueDeck).

#### Token Counting

- **`tiktoken-rs`** (v0.5): OpenAI's tokenizer (cl100k_base for GPT-3.5/4).
  - **Why Not**: Manual heuristics (inaccurate), `tokenizers` (too heavy).
  - **Accuracy**: Matches OpenAI's exact token count for budget enforcement.

#### File System

- **`walkdir`** (v2.5): Recursive directory traversal.
  - **Alternative**: `ignore` crate (faster but pulls in `.gitignore` parsing).
  
- **`globset`** (v0.4): `.gitignore`-style pattern matching.
  - **Use Case**: Exclude patterns in config (`exclude_patterns`).

- **`notify`** (v6.1): Cross-platform file watcher for `cue watch`.
  - **Platform**: Uses `inotify` (Linux), `FSEvents` (macOS), `ReadDirectoryChangesW` (Windows).

#### Hashing

- **`sha2`** (v0.10): SHA-256 for cache invalidation.
  - **Why Not**: `blake3` (faster but not needed for cache use case), `md5` (insecure).

#### Configuration

- **`config`** (v0.14): Hierarchical config with cascading (file → env → defaults).
  - **Supports**: TOML, YAML, JSON, environment variables.

#### User Interface

- **`skim`** (v0.10): Fuzzy finder (`fzf` clone in Rust) for `cue open`.
  - **Why Not**: Shell out to `fzf` (external dependency), `inquire` (no fuzzy matching).

- **`arboard`**: Cross-platform clipboard library.
  - **Use Case**: Copy `SCENE.md` to clipboard after `cue scene` or `cue watch`.

#### Logging & Telemetry

- **`tracing`** (v0.1): Structured logging with spans.
  - **Why Not**: `log` (unstructured, less powerful), `slog` (more verbose).
  - **Features**: Async-aware, context propagation for distributed tracing.

- **`tracing-subscriber`** (v0.3): Log formatting and filtering.
  - **Features**: `env-filter` for `RUST_LOG=debug` control.

- **`tracing-appender`** (v0.2): Log file rotation and appending.
  - **Use Case**: Rotate log files in `.cuedeck/logs/`.

#### Graph & Search

- **`petgraph`** (v0.6): Graph data structures and algorithms.
  - **Use Case**: `DependencyGraph` for reference resolution and cycle detection.
  - **Why Not**: Manual implementation (error-prone), `daggy` (less features).

- **`regex`** (v1.10): Regular expression engine.
  - **Use Case**: Secret pattern matching for masking API keys.
  - **Performance**: Highly optimized, supports lazy compilation.

#### Performance

- **`rayon`** (v1.10): Data parallelism library.
  - **Use Case**: Parallel file parsing for cold start optimization.
  - **Features**: Work-stealing thread pool, zero-cost abstractions.

- **`memmap2`** (v0.9): Memory-mapped file I/O.
  - **Use Case**: Efficient reading of large documentation files.
  - **Benefit**: OS-managed caching, reduced memory copies.

#### Terminal User Interface (Phase 8)

- **`ratatui`** (v0.26): Terminal UI framework.
  - **Why Not**: `tui-rs` (unmaintained, ratatui is the successor), `cursive` (less flexible).
  - **Use Case**: Interactive TUI dashboard (`cue tui` command).
  - **Features**: Immediate mode rendering, composable widgets, buffer-based rendering.

- **`crossterm`** (v0.27): Cross-platform terminal control.
  - **Why Not**: `termion` (Unix-only), `pancurses` (heavier).
  - **Use Case**: Keyboard input handling, terminal setup/cleanup.
  - **Platform**: Uses native APIs (Windows Console API, Unix termios).

- **`tui-textarea`** (v0.4): Multi-line text input widget.
  - **Use Case**: Optional, for search input in TUI.
  - **Alternative**: Custom text input implementation if not needed.

## Version Pinning Strategy

```toml
# workspace.dependencies in root Cargo.toml
# Use ^version (compatible updates) for API-stable crates
clap = "^4.5"        # OK: clap v4 is stable
tokio = "^1.36"      # OK: tokio v1 has strong SemVer guarantees

# Use =version (exact pin) for breaking-change-prone crates
gray_matter = "=0.2"  # WARNING: v0.x can break on minor bumps
```

**Update Policy**:

- **Security patches**: Immediate update (e.g., `tokio` vulnerability).
- **Minor versions**: Quarterly review for new features.
- **Major versions**: Only when feature-critical (minimize churn).

> **Note**: The full dependency list with versions is in [IMPLEMENTATION_TEMPLATES.md](../03_agent_design/IMPLEMENTATION_TEMPLATES.md).

---
**Related Docs**: [SYSTEM_ARCHITECTURE.md](./SYSTEM_ARCHITECTURE.md), [IMPLEMENTATION_TEMPLATES.md](../03_agent_design/IMPLEMENTATION_TEMPLATES.md)
