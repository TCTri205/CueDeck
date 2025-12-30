# Module Design Specification (Rust)

This document defines the Structs, Enums, and Traits for the core crates.

## 1. `cue_core` (The Brain)

### Constants

```rust
pub const TOKEN_LIMIT_DEFAULT: usize = 32_000;
pub const WORKSPACE_DIR: &str = ".cuedeck";
```

### Crate Dependency Graph

```mermaid
graph TD
    CLI[cue_cli] --> Core[cue_core]
    CLI --> Config[cue_config]
    CLI --> Common[cue_common]
    
    MCP[cue_mcp] --> Core
    MCP --> Common
    
    Core --> Config
    Core --> Common
    
    Config --> Common
    
    style Common fill:#f9f,stroke:#333 -- Foundation
```

### Structs

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub path: PathBuf,
    pub hash: String, // SHA256 hex
    pub anchors: Vec<Anchor>,
    pub frontmatter: Option<serde_yaml::Value>,
    pub tokens: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Anchor {
    pub slug: String, // e.g., "Login Flow" -> "login-flow"
    pub header: String, // Original Text
    pub level: u8, // 1-6
    pub start_line: usize,
    pub end_line: usize,
}

pub struct Workspace {
    pub root: PathBuf,
    pub config: Config,
    pub cache: CacheManager,
    pub dag: Graph,
}
```

### Traits

```rust
pub trait ContextSource {
    fn resolve(&self, query: &str) -> Result<Vec<Snippet>>;
    fn content(&self, path: &Path, range: Option<Range>) -> Result<String>;
}
```

## 2. `cue_config` (Settings)

### Config Structs

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub core: CoreConfig,
    pub parser: ParserConfig,
    pub security: SecurityConfig,
    pub mcp: McpConfig,
}

// Sub-structs mirror the TOML structure in CONFIGURATION_REFERENCE.md
```

## 3. `cue_mcp` (The Server)

### Enums

```rust
#[derive(Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum McpRequest {
    #[serde(rename = "read_context")]
    ReadContext { query: String, limit: Option<usize> },
    
    #[serde(rename = "read_doc")]
    ReadDoc { path: String, anchor: Option<String> },

    #[serde(rename = "list_tasks")]
    ListTasks { status: Option<String> },

    #[serde(rename = "update_task")]
    UpdateTask { id: String, updates: serde_json::Value },
}
```

## 3. `cue_cli` (The Interface)

### Clap Commands

```rust
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new CueDeck workspace
    Init,
    /// Open the Fuzzy Finder for context
    Open { query: Option<String> },
    /// Generate the full SCENE.md context
    Scene {
        #[arg(long, short)]
        dry_run: bool,
    },
    /// Start the file watcher daemon
    Watch,
    /// Check workspace health
    Doctor,
    /// Manage Task Cards
    Card {
        #[command(subcommand)]
        cmd: CardCommands,
    },
    /// Clean cache directory
    Clean,
}

#[derive(Subcommand)]
pub enum CardCommands {
    /// Create a new task card
    New { title: String },
}
```

---
**Related Docs**: [SYSTEM_ARCHITECTURE.md](./SYSTEM_ARCHITECTURE.md), [API_DOCUMENTATION.md](../04_tools_and_data/API_DOCUMENTATION.md)
