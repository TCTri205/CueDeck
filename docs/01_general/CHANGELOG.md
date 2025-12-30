# Changelog

All notable changes to CueDeck will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- Initial documentation structure.

---

## [2.1.0] - 2024-XX-XX (Target Release)

### Added

- **Rust Workspace Architecture**: Modularized into `cue_common`, `cue_config`, `cue_core`, `cue_cli`, `cue_mcp`.
- **Incremental Parser**: SHA256-based change detection for <5ms hot-path.
- **Lazy Garbage Collection**: Self-healing cache that purges stale entries on access.
- **Granular Anchor Resolution**: Extract specific sections using `@path#Header` syntax.
- **Token Budgeting**: Greedy pruning algorithm to fit LLM context windows.
- **MCP Server (`cue_mcp`)**: JSON-RPC 2.0 over Stdio with strict I/O isolation.
- **Secret Masking**: Regex-based guard to prevent API key leakage.
- **CLI Commands**: `cue init`, `cue open`, `cue scene`, `cue watch`, `cue doctor`, `cue clean`.
- **TUI**: Interactive fuzzy finder using `skim`.

### Security

- Default patterns for OpenAI, GitHub, AWS, and Slack secrets.

---

## [2.0.0] - (Hypothetical Previous Version)

### Changed

- Migrated from TypeScript to Rust for performance gains.

---
**Related Docs**: [ROADMAP.md](./ROADMAP.md), [PROJECT_OVERVIEW.md](./PROJECT_OVERVIEW.md)
