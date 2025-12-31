# Contributing to CueDeck

**New to CueDeck?** Start with [QUICK_START.md](./QUICK_START.md) to get set up in 10 minutes.

---

## 1. Environment Setup

CueDeck is a standard Rust project. You need `rustup` and `cargo` installed.

### Prerequisites

- **Rust**: Stable (latest).
- **Tools**:
  - `cargo-edit` (optional, for managing deps).
  - `cargo-insta` (required for snapshot testing).
  - `cargo-nextest` (recommended for faster tests).

### Setup Command

```bash
# Clone repository (thay <YOUR_ORG> bằng org thực tế)
git clone https://github.com/<YOUR_ORG>/cuedeck.git
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

### PR Template

When creating a pull request, include:

```markdown
## Description
Brief summary of changes and motivation.

## Type of Change
- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to change)
- [ ] Documentation update

## Testing
- [ ] Unit tests pass (`cargo test --workspace`)
- [ ] Snapshot tests pass (`cargo insta test`)
- [ ] Manual testing completed

## CI Checklist
- [ ] Code formatted (`cargo fmt --all -- --check`)
- [ ] No clippy warnings (`cargo clippy --workspace -- -D warnings`)
- [ ] All tests pass (`cargo nextest run`)
- [ ] Documentation updated (if applicable)
- [ ] JSON schemas validated (if schemas changed)
```

### CI Fail Conditions

Your PR will be blocked if:

- ❌ Any test fails
- ❌ Code not formatted  
- ❌ Clippy warnings exist
- ❌ Coverage drops below threshold
- ❌ `cue doctor` finds issues
- ❌ Broken doc links

See [TESTING_STRATEGY.md](../05_quality_and_ops/TESTING_STRATEGY.md#ci-fail-conditions) for details.

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

### Implementing Core Features

When adding features to `cue_core`:

1. **Types**: Define new types in `cue_common/src/types.rs`.
2. **Errors**: Add error variants to `cue_common/src/errors.rs`.
3. **Logic**: Implement in appropriate `cue_core/src/*.rs` module.
4. **Test**: Add unit tests in the same file + integration tests in `tests/`.
5. **Docs**: Update `MODULE_DESIGN.md` and `API_DOCUMENTATION.md`.

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
