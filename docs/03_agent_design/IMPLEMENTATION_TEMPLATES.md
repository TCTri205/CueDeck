# Implementation Templates

This document provides copy-pasteable starter files for the CueDeck workspace.

---

## 1. Workspace `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "crates/cue_common",
    "crates/cue_config",
    "crates/cue_core",
    "crates/cue_cli",
    "crates/cue_mcp",
]

[workspace.package]
version = "2.1.0"
edition = "2021"
license = "MIT"
authors = ["CueDeck Team"]
repository = "https://github.com/your-org/cuedeck"

[workspace.dependencies]
# CLI & Args
clap = { version = "4.5", features = ["derive", "string"] }
# Async Runtime
tokio = { version = "1.36", features = ["full", "tracing"] }
# Error Handling
miette = { version = "7.2", features = ["fancy"] }
thiserror = "1.0.57"
# Serialization
serde = { version = "1.0.197", features = ["derive"] }
serde_json = "1.0.114"
serde_yaml = "0.9.33"
# Parsing
pulldown-cmark = "0.10"
gray_matter = "0.2"
tiktoken-rs = "0.5"
# File System
walkdir = "2.5"
globset = "0.4"
notify = "6.1"
# Hashing
sha2 = "0.10"
# Config
config = "0.14"
# UI
skim = "0.10"
# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
# Clipboard
arboard = "3.3"
# Testing
insta = { version = "1.34", features = ["yaml"] }
tempfile = "3.10"
criterion = "0.5"
```

---

## 2. `.gitignore`

```gitignore
# Rust Build Artifacts
/target/
**/*.rs.bk
Cargo.lock

# CueDeck Runtime
.cuedeck/.cache/
.cuedeck/logs/
.cuedeck/SCENE.md

# IDE
.idea/
.vscode/
*.swp

# OS
.DS_Store
Thumbs.db

# Secrets (should never be committed)
.env
*.pem
```

---

## 3. GitHub Actions CI (`.github/workflows/ci.yml`)

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Clippy lint
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Build
        run: cargo build --release --workspace
      - name: Run tests
        run: cargo test --workspace
      - name: Run benchmarks (optional)
        run: cargo bench --no-run

  msrv:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.75 # MSRV
      - run: cargo check --workspace
```

---

## 4. Crate `Cargo.toml` Template (e.g., `crates/cue_common/Cargo.toml`)

```toml
[package]
name = "cue_common"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true
miette.workspace = true

[dev-dependencies]
insta.workspace = true
```

---

---

## 5. CLI Entry Point (`crates/cue_cli/src/main.rs`)

```rust
use clap::Parser;
use miette::{IntoDiagnostic, Result};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Init,
    Doctor,
    Watch,
    Scene {
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("Initializing workspace...");
            // cue_core::init::run()?;
        }
        Commands::Doctor => {
            println!("Running health checks...");
            // cue_core::doctor::run()?;
        }
        _ => {
            println!("Command not yet implemented!");
        }
    }

    Ok(())
}
```

---

## 6. Core Lib Skeleton (`crates/cue_core/src/lib.rs`)

```rust
pub mod config;
pub mod dag;
pub mod parser;
pub mod session;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum CueError {
    #[error("Feature not implemented")]
    #[diagnostic(code(cuedeck::core::todo))]
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, CueError>;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

---
**Related Docs**: [TECH_STACK.md](../02_architecture/TECH_STACK.md), [PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md), [CONTRIBUTING.md](../01_general/CONTRIBUTING.md)
