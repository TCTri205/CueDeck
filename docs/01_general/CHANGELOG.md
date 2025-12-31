# Changelog

All notable changes to CueDeck will be documented in this file.

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
