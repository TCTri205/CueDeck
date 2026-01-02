# Changelog

All notable changes to CueDeck will be documented in this file.

## [2.5.0] - 2026-01-01

### Performance

- **Binary Size Optimization**: Reduced release binary from **29.3 MB to 24.1 MB** (17.5% reduction)
  - Applied aggressive compiler optimizations:
    - `opt-level = "z"` - Optimize for size instead of speed
    - `lto = "fat"` - Full link-time optimization across all crates
    - `panic = "abort"` - Smaller panic handler (no unwinding)
    - `strip = "symbols"` - Remove all debug symbols
  - Created optional feature flags for `fastembed` and `petgraph`
  - Analyzed dependencies with `cargo tree --duplicate`
  - Binary remains fully functional with all features enabled

### Documentation

- Added comprehensive installation guide to README.md
  - Pre-built binary installation instructions
  - Build from source guide with requirements
  - MCP integration setup for Antigravity IDE and Claude Desktop
  - Quick start guide with essential commands
  - Troubleshooting section for common issues
- Created QA test report documenting 23/23 passing tests
- Enhanced README with features showcase and project structure

## [2.4.0] - 2026-01-01

### Added

- **CueEngine**: Stateful architecture for incremental updates (43x faster hot init)
- **Self-Updater**: `cue upgrade` command with automatic binary updates from GitHub
- **Progress Indicators**: Spinners for `cue scene` and `cue watch` operations
- **Parallel Processing**: Parallelized file parsing and semantic search with `rayon`
- **Document Cache**: SHA256-based caching with persistence and auto-repair
- **CI/CD Pipeline**: Automated multi-platform builds via GitHub Actions

### Performance

- Hot initialization: 1.55s → 35ms (43x speedup)
- Scene rendering: <20ms from memory
- Watch mode: Incremental updates only re-parse changed files
- Binary size: Reduced ~35% via strip optimization

### Changed

- `cue watch` now maintains state in memory for faster updates
- `generate_scene` refactored to use `CueEngine` internally
- Release builds optimized with LTO and strip

### Technical

- Added dependencies: `rayon`, `bincode`, `schemars`, `self_update`, `indicatif`
- Enhanced `DependencyGraph` with incremental update methods
- Implemented read-only cache access for rendering
- CI/CD: GitHub Actions with multi-platform release automation

## [2.3.0] - 2025-12-31

### Added

- 🔗 **Advanced Graph Features** with `petgraph` integration
- 📊 `cue graph` command for dependency visualization
  - ASCII format (terminal output)
  - Mermaid format (GitHub/docs integration)
  - DOT format (Graphviz compatibility)
  - JSON export (machine-readable)
- 📈 Graph statistics (`--stats` flag)
  - Node and edge counts
  - Cycle detection with path tracing
  - Orphan document identification
- 🔍 Enhanced `cue doctor` with real cycle detection
  - Full dependency path for cycles
  - Improved error reporting

### Changed

- ⬆️ Added `petgraph` v0.6 dependency
- 🔧 Extended `DependencyGraph` with analytics methods
- 📚 Updated `CLI_REFERENCE.md` with graph commands

### Testing

- ✅ 39/39 tests passing (5 new graph tests)
- ✅ Manual verification of all graph formats
- ✅ Cycle detection validated on test graphs

## [2.2.0] - 2025-12-31

### Added

- ✨ Semantic search with FastEmbed-rs (all-MiniLM-L6-v2)
- 🎯 `--semantic` flag for `cue open` command
- 🔧 MCP `semantic` parameter for `read_context` tool
- 📦 New `embeddings.rs` module with lazy model initialization
- 🧪 Integration tests for semantic search functionality

### Changed

- ⬆️ Upgraded `fastembed` to v4.9.1 for Rust compatibility
- 🔄 Extended `search_workspace()` with semantic parameter
- 📚 Updated documentation with semantic search usage

### Performance

- First run: ~5s (model download ~22MB + initialization)
- Subsequent searches: 10-15s for full workspace scan
- Keyword search: <100ms (unchanged)
- Model cached after first use

### Testing

- ✅ 29/29 tests passing
- ✅ Full integration test suite
- ✅ Semantic vs keyword search comparison tests

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [2.1.0] - 2025-12-31

### Added (New Features)

- **Context Search**: `cue open` now uses fuzzy search to find files and content instantly.
- **Real-time Watcher**: `cue watch` auto-regenerates the scene and updates the clipboard on file changes.
- **MCP Server**: Full implementation of `read_context`, `read_doc`, `list_tasks`, `create_task`, `update_task`.
- **Core Logic**: Robust Frontmatter parsing, Link detection, Cycle detection, and Graph resolution.
- **Task Management**: Create, Update, and List tasks via CLI and MCP.
- **CLI Refactor**: Unified `cue_core` logic for all commands.
- **Self-Update**: `cue upgrade` checks for new GitHub releases.
- **Security**: Secret masking for API keys in all outputs.

### Documentation

- Complete documentation suite (Architecture, Tools Spec, CLI Reference).

### Fixed

- **Graph Resolution**: Correctly handles circular dependencies.
- **Parsing**: Robust regex for various frontmatter formats.

---

## [2.0.0] - (Hypothetical Previous Version)

### Changed

- Migrated from TypeScript to Rust for performance gains.

---
**Related Docs**: [ROADMAP.md](./ROADMAP.md), [PROJECT_OVERVIEW.md](./PROJECT_OVERVIEW.md)
