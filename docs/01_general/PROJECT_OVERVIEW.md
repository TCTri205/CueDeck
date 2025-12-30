# Project Overview: CueDeck (Rust Ecosystem Edition)

> **Version**: 2.1.0
> **Architecture**: Rust Workspace (Monorepo)
> **Model**: Local-First Ecosystem + Native MCP Server

## 1. Goal

To build a **high-performance Project Context Management Tool** (<5ms hot-path) that integrates deeply into DevOps workflows and AI Coding environments (like Cursor or Local LLMs via MCP).
The system is designed to be the "Hippocampus" (Memory Center) for AI Agents, managing context with granular precision, self-healing capability, and intelligent resource management.

## 2. Core Value Proposition

- **Extreme Performance**: Built on Rust, targeting <5ms for hot-path incremental updates.
- **Intelligent Context**:
  - **Incremental Parser**: Only re-processes changed files (SHA256 diff).
  - **DAG Resolution**: Graph-based dependency management to prevent circular refs.
  - **Token Budgeting**: Smart pruning of context to fit LLM windows.
- **Ecosystem Integration**:
  - **Native MCP Server**: Exposes context via standard `stdio` JSON-RPC 2.0.
  - **Developer Experience**: CLI and TUI (`skim`) for human interaction.
- **Safety & Security**:
  - **Secret Guard**: Regex-based scanning to mask API keys before they leave the local machine.
  - **Cycle Detection**: Prevents infinite loops in document references.

## 3. Scope

**What CueDeck Does (The "Brain"):**

- **Manages**: A Rust Workspace treated as a "Knowledge Graph".
- **Watches**: Real-time file monitoring (`notify`) with immediate consistency.
- **Serves**: JSON-RPC endpoints for AI Agents to `read` and `search` context.
- **Builds**: "Scenes" (`SCENE.md`) which are fully resolved, pruned, and secure snapshots of the project.

**What CueDeck Does NOT Do:**

- It is NOT a Code Generator (It provides the *context* for generation).
- It is NOT a generalized Vector Database (It uses "Graph + Search" architecture).
- It is NOT a cloud service (It is 100% Local-First).

## 4. Target Audience

## 5. Directory Map (Where to find things)

| Directory | Purpose | Key Files |
| :--- | :--- | :--- |
| **`crates/cue_core`** | The "Brain" (Rust) | `parser.rs`, `dag.rs` |
| **`crates/cue_cli`** | The "Face" (TUI) | `main.rs`, `commands/` |
| **`crates/cue_mcp`** | The "Voice" (AI) | `server.rs`, `router.rs` |
| **`docs/`** | The "Manual" | `SYSTEM_ARCHITECTURE.md`, `TOOLS_SPEC.md` |
| **`.cuedeck/cards`** | The "Work" | `2a9f1x.md` (Active), `8b2c4z.md` (Done) |
| **`New!`** | **Guides** | [`CONTRIBUTING.md`](./CONTRIBUTING.md), [`TROUBLESHOOTING.md`](../05_quality_and_ops/TROUBLESHOOTING.md) |

---
**Related Docs**: [GLOSSARY.md](./GLOSSARY.md), [ROADMAP.md](./ROADMAP.md), [SYSTEM_ARCHITECTURE.md](../02_architecture/SYSTEM_ARCHITECTURE.md)
