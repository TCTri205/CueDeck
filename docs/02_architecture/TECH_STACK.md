# Technology Stack

## Core Language

- **Rust**: Selected for performance, safety, and workspace capabilities.

## Key Crates (Libraries)

### Core Dependencies (`Cargo.toml`)

```toml
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
```

### Justification

- **`tokio`**: We use the `full` feature set because we need the Multi-threaded Scheduler for the `cue_mcp` server to handle concurrent requests without blocking.
- **`miette`**: Chosen over `anyhow` because `cue` is a user-facing tool, and `miette` provides beautiful, colorful error diagnostics with source code spans.
