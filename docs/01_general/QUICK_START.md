# Quick Start Guide

Get CueDeck running in under 10 minutes.

## Prerequisites

- **Rust**: 1.75.0+ (`rustup` recommended) — MSRV for `async fn` in traits
- **Git**: For cloning the repository
- **Editor**: VS Code, Zed, or your preferred IDE

## 1. Clone and Build

```bash
# Clone the repository (thay <YOUR_USERNAME> bằng username thực tế)
git clone https://github.com/<YOUR_USERNAME>/cuedeck.git
cd cuedeck

# Build in release mode
cargo build --release

# Verify installation
./target/release/cue --version
# Expected: cuedeck 2.1.0
```

## 2. Initialize Your First Workspace

```bash
# Create a test project
mkdir ~/my-project
cd ~/my-project

# Initialize CueDeck
../cuedeck/target/release/cue init

# Verify structure
ls -la .cuedeck/
# Expected: cards/, docs/, config.toml, .cache/
```

## 3. Create Your First Card

```bash
# Create a task card
cue card new "Add authentication"

# View generated file
cat .cuedeck/cards/*.md
```

**Expected Output:**

```markdown
---
id: "2a9f1x"
title: "Add authentication"
status: "todo"
priority: "medium"
---

# Add authentication

[Your task description here]
```

## 4. Generate a Scene

```bash
# Generate context snapshot
cue scene

# View output
cat .cuedeck/SCENE.md
```

The scene will contain your active cards and any referenced docs.

## 5. Run Tests

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p cue_core

# Run with output
cargo test -- --nocapture
```

## 6. Make Your First Change

### Example: Add a new CLI command

1. **Add to CLI enum** (`crates/cue_cli/src/main.rs`):

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands
    /// Show workspace statistics
    Stats,
}
```

1. **Implement handler** (`crates/cue_cli/src/commands/stats.rs`):

```rust
pub fn run_stats() -> Result<()> {
    println!("Workspace statistics:");
    // Implementation here
    Ok(())
}
```

1. **Test it**:

```bash
cargo run -- stats
```

## 7. Development Workflow

```bash
# Watch for changes and auto-rebuild
cargo watch -x "build --release"

# Run with debug logging
RUST_LOG=debug cargo run -- scene

# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets
```

## Next Steps

- Read [CONTRIBUTING.md](./CONTRIBUTING.md) for PR guidelines
- Explore [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md) for architecture
- Check [ROADMAP.md](./ROADMAP.md) for feature priorities

---
**Related Docs**: [PROJECT_OVERVIEW.md](./PROJECT_OVERVIEW.md), [CONTRIBUTING.md](./CONTRIBUTING.md)
