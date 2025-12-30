# Contributing to CueDeck

## 1. Development Environment Setup

CueDeck is a standard Rust project. You need `rustup` and `cargo` installed.

### Prerequisites

- **Rust**: Stable (latest).
- **Tools**:
  - `cargo-edit` (optional, for managing deps).
  - `cargo-insta` (required for snapshot testing).
  - `cargo-nextest` (recommended for faster tests).

### Setup Command

```bash
# Clone repository
git clone https://github.com/your-org/cuedeck.git
cd cuedeck

# Install dev tools
cargo install cargo-insta

# Verify setup
cargo test --workspace
```

## 2. Coding Standards

We enforce strict quality gates using `clippy` and `rustfmt`.

### Style Guide

- **Async**: Use `tokio::main` for binaries, `async fn` for IO-bound libs.
- **Errors**: Use `miette::Result` for application code, `thiserror` for library code.
- **Paths**: Always use `PathBuf`, never `String` for file paths.

### Pre-Commit Checks

Before submitting a PR, run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

## 3. Pull Request Process

1. **Branch Naming**: `feat/your-feature`, `fix/your-bug`, `docs/doc-update`.
2. **Conventional Commits**: e.g., `feat(parser): add separate frontmatter parsing`.
3. **Review**: At least one approval required. All CI checks must pass.

## 4. How-To Guides

### Add a New CLI Command

1. **Edit `crates/cue_cli/src/main.rs`**: Add variant to `Commands` enum.
2. **Edit `crates/cue_cli/src/commands/`**: Create new module (e.g., `my_cmd.rs`).
3. **Register**: Impl `run_my_cmd()` and call it in `main` match arm.
4. **Test**: Add snapshot test in `tests/cli.rs`.

### Add a New MCP Tool

1. **Edit `crates/cue_mcp/src/lib.rs`**: Add variant to `McpRequest` enum (e.g., `NewTool`).
2. **Edit `crates/cue_mcp/src/router.rs`**: Add Logic in `handle_request()`.
3. **Spec**: Update `docs/04_tools_and_data/TOOLS_SPEC.md` with JSON Schema.

## 5. Release Process

To ship a new version (e.g., `v2.1.0`):

1. **Bump Version**: Update `Cargo.toml` in all crates and workspace root.
2. **Changelog**: Move `[Unreleased]` to `[2.1.0] - YYYY-MM-DD`.
3. **Tag**:

    ```bash
    git tag v2.1.0
    git push origin v2.1.0
    ```

4. **Publish**: CI will automatically build binaries and creating a GitHub Release.
5. **Crates.io**:

    ```bash
    cargo publish -p cue_common
    cargo publish -p cue_config
    # ... (in dependency order)
    ```

---
**Related Docs**: [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md), [TESTING_STRATEGY.md](../05_quality_and_ops/TESTING_STRATEGY.md)
