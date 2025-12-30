# Project Roadmap

## Phase 1: Foundation (Week 1)

**Theme**: Workspace Setup & Configuration Infrastructure

- [ ] **Workspace**: Initialize `cuedeck-workspace` with Cargo members: `cue_common`, `cue_config`, `cue_core`, `cue_cli`, `cue_mcp`.
- [ ] **Config Engine**: Implement Cascadia-style config loading (`native_config` + `toml` + CLI args) in `cue_config`.
- [ ] **Core Types**: Define `CueError` (Miette), `Document`, `Card`, and `Anchor` types in `cue_common`.
- [ ] **Basic I/O**: Implement safe file reading and SHA256 hashing helpers.

> [!CHECK] Phase 1 Exit Criteria
>
> - `cargo build` passes for all 5 crates.
> - `cargo test` passes for `cue_config` (loading) and `cue_common` (types).
> - `cue_cli` runs `cue --help` successfully.

## Phase 2: The Core Brain (Weeks 1-2)

**Theme**: Intelligent Data Processing

- [ ] **Caching System**:
  - Implement `CacheManager` to load/save `.cuedeck/.cache/metadata.json`.
  - Implement **Lazy GC** logic (invalidate on miss).
- [ ] **Parser Engine**:
  - Implement Markdown AST parsing via `pulldown-cmark`.
  - Extract `frontmatter` using `gray_matter`.
  - Implement **Anchor Extraction** algorithm (Header-based segmentation).
- [ ] **Graph Theory**:
  - Implement `DependencyGraph` to resolve `@ref` links.
  - Implement **Cycle Detection** algorithm (DFS/Tarjan's).

## Phase 3: CLI & Experience (Weeks 2-3)

**Theme**: Human Interaction

- [ ] **Interactive UI**: Integrate `skim` for `cue open` (fuzzy finding cards/docs).
- [ ] **Scene Builder**: Connect Graph + Cache to generate fully resolved `SCENE.md`.
- [ ] **Watcher**: Implement `cue watch` using `notify`:
  - Event -> Debounce (500ms) -> Re-Calc -> Clipboard (`arboard`).
- [ ] **CLI Commands**: Finalize `cue init`, `cue card new`, `cue doctor`.

## Phase 4: MCP & Polish (Weeks 3-4)

**Theme**: AI Integration & Distribution

- [ ] **MCP Server (`cue_mcp`)**:
  - Implement JSON-RPC 2.0 Loop over `stdin`/`stdout`.
  - **Crucial**: Route all logs to `stderr` or file to prevent protocol corruption.
- [ ] **Tool Implementation**: `read_context`, `read_doc`, `list_tasks`, `update_task`.
- [ ] **Security**: Finalize Secret Masking (Regex Guard).
- [ ] **DevOps**: Setup GitHub Actions for `cargo build --release` and `cargo-binstall` support.

---
**Related Docs**: [PROJECT_STRUCTURE.md](../03_agent_design/PROJECT_STRUCTURE.md), [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md), [TESTING_STRATEGY.md](../05_quality_and_ops/TESTING_STRATEGY.md)
