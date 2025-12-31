# Development Guide

This guide provides practical instructions for setting up your development environment, debugging, and profiling CueDeck.

## 1. IDE Setup (VSCode)

We recommend **VSCode** with the following extensions:

### Extensions

- **rust-analyzer**: Official Rust language support (Critical).
- **crates**: Helps manage dependency versions in `Cargo.toml`.
- **Even Better TOML**: Syntax highlighting for TOML files.
- **Error Lens**: Inline error display (Optional but recommended).

### Settings (`.vscode/settings.json`)

```json
{
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

## 2. Local Development Workflow

### Running the Watcher

To test the file watcher loop without building a release binary:

```bash
# Terminal 1: Run the watcher on the current directory
RUST_LOG=info cargo run -- watch .

# Terminal 2: Modify a file to trigger the watcher
touch test.md
```

### Running the MCP Server

To test the MCP integration locally:

```bash
# Run server (connects to stdio)
cargo run -- mcp
```

> **Tip**: Use an MCP inspector or client to send JSON-RPC messages via stdin.

### Running Specific Tests

To run only the watcher integration tests:

```bash
cargo test --test watcher_integration
```

## 3. Debugging

### Using `dbg!`

The quickest way to inspect values. Note that `dbg!` output goes to `stderr`, which is safe for MCP mode.

```rust
let config = dbg!(Config::load()?);
```

### LLDB (VSCode)

Use the "CodeLLDB" extension for breakpoints.

**.vscode/launch.json**:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug CLI",
            "program": "${workspaceFolder}/target/debug/cue",
            "args": ["scene"],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

## 4. Profiling

### Trace Logging

Enable verbose logs to trace execution flow:

```bash
RUST_LOG=trace cargo run -- scene
```

### Flamegraphs regarding Performance

See [PERFORMANCE_OPTIMIZATION.md](../02_architecture/PERFORMANCE_OPTIMIZATION.md) for detailed instructions on using `cargo-flamegraph`.

## 5. Common Tasks

### Adding a New CLI Command

1. Create `crates/cue_cli/src/commands/new_cmd.rs`.
2. Register it in `crates/cue_cli/src/commands/mod.rs`.
3. Add the subcommand enum in `crates/cue_cli/src/main.rs`.

### Adding a New MCP Tool

1. Define the tool schema in `crates/cue_mcp/src/tools.rs`.
2. Implement the handler in `crates/cue_mcp/src/router.rs`.
3. Update [TOOLS_SPEC.md](../04_tools_and_data/TOOLS_SPEC.md).

## 6. Environment Variables

| Variable | Default | Description |
|:---------|:--------|:------------|
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `CUEDECK_CONFIG` | `.cuedeck/config.toml` | Override config path |
| `CUEDECK_NO_COLOR` | `false` | Disable colored output |
| `EDITOR` | System default | Editor for `cue open` |

### Example Usage

```bash
# Verbose debugging
RUST_LOG=debug cargo run -- scene

# Custom config for testing
CUEDECK_CONFIG=/tmp/test-config.toml cue doctor

# CI mode (no colors)
CUEDECK_NO_COLOR=1 cue doctor
```

## 7. Build Configurations

### Debug Build (Development)

```bash
cargo build                    # Fast compile, slow runtime
cargo run -- scene             # Run directly
```

### Release Build (Production)

```bash
cargo build --release          # Slow compile, fast runtime
./target/release/cue scene     # Optimized binary
```

### Build Profiles

| Profile | Optimizations | Debug Info | Use Case |
|:--------|:-------------|:-----------|:---------|
| `dev` | 0 | Full | Development |
| `release` | 3 | None | Distribution |
| `test` | 0 | Full | Testing |
| `bench` | 3 | Limited | Benchmarking |

## 8. Cross-Platform Considerations

### Windows

- Use forward slashes `/` in paths within code (Rust handles conversion)
- Test with PowerShell and CMD
- Clipboard: Uses Windows API via `arboard`

### Linux

- Increase `inotify` watches for large projects:

  ```bash
  echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
  sudo sysctl -p
  ```

- Clipboard: Requires `xclip` or `wl-copy`

### macOS

- Remove quarantine flag after download:

  ```bash
  xattr -d com.apple.quarantine cue
  ```

## 9. Useful Aliases

Add to your shell profile for faster development:

```bash
# Build and run tests
alias ct="cargo test --workspace"

# Run with debug logging
alias cd-debug="RUST_LOG=debug cargo run --"

# Format and lint
alias cd-check="cargo fmt --all && cargo clippy --workspace -- -D warnings"

# Watch mode development
alias cd-watch="cargo watch -x 'build --release'"
```

---
**Related Docs**: [CONTRIBUTING.md](./CONTRIBUTING.md), [PERFORMANCE_OPTIMIZATION.md](../02_architecture/PERFORMANCE_OPTIMIZATION.md), [QUICK_START.md](./QUICK_START.md)
