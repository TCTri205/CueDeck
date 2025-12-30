# Project Structure

## Source Code Tree (Planned)

```text
cuedeck-workspace/
├── Cargo.toml                     # Workspace definition
├── .github/
│   └── workflows/
│       └── ci.yml                 # GitHub Actions CI
│
├── crates/
│   ├── cue_common/               # FOUNDATION CRATE
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── errors.rs          # CueError enum (miette)
│   │       └── types.rs           # Document, Anchor, Card structs
│   │
│   ├── cue_config/               # CONFIGURATION CRATE
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── loader.rs          # Cascading config logic
│   │
│   ├── cue_core/                 # BRAIN CRATE
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs          # Markdown AST + Frontmatter
│   │       ├── dag.rs             # Reference Graph + Cycle Detection
│   │       ├── cache.rs           # CacheManager + Lazy GC
│   │       ├── search.rs          # Fuzzy Search (skim)
│   │       ├── scene.rs           # Scene Builder + Token Pruning
│   │       └── security.rs        # Secret Masking
│   │
│   ├── cue_cli/                  # CLI/TUI CRATE
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs            # Clap entry point
│   │       └── commands/
│   │           ├── mod.rs
│   │           ├── init.rs        # cue init
│   │           ├── open.rs        # cue open (skim TUI)
│   │           ├── scene.rs       # cue scene
│   │           ├── watch.rs       # cue watch (notify)
│   │           ├── doctor.rs      # cue doctor
│   │           ├── card.rs        # cue card new
│   │           └── clean.rs       # cue clean
│   │
│   └── cue_mcp/                  # MCP SERVER CRATE
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── server.rs          # Tokio stdin/stdout loop
│           └── router.rs          # JSON-RPC dispatch
│
└── tests/                         # Integration Tests
    ├── snapshot_scene.rs          # Insta snapshots
    └── watcher_integration.rs     # tempfile + tokio tests
```

## Key Files by Purpose

| Purpose | Primary File | Backup/Related |
| :--- | :--- | :--- |
| Error Definitions | `cue_common/src/errors.rs` | `ERROR_HANDLING_STRATEGY.md` |
| Core Types | `cue_common/src/types.rs` | `MODULE_DESIGN.md` |
| Config Loading | `cue_config/src/loader.rs` | `KNOWLEDGE_BASE_STRUCTURE.md` |
| DAG Logic | `cue_core/src/dag.rs` | `ALGORITHMS.md` |
| MCP Router | `cue_mcp/src/router.rs` | `TOOLS_SPEC.md`, `EXAMPLES.md` |
| CLI Commands | `cue_cli/src/commands/*.rs` | `CLI_REFERENCE.md`, `CLI_UX_FLOWS.md` |

---
**Related Docs**: [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md), [ROADMAP.md](../01_general/ROADMAP.md), [TECH_STACK.md](../02_architecture/TECH_STACK.md)
