# Error Handling Strategy

## 1. Philosophy

CueDeck is a **user-facing developer tool**. Errors must be:

1. **Instructional**: Tell the user *how* to fix it, not just what broke.
2. **Structured**: Machine-readable (JSON-RPC) for MCP, pretty-printed for CLI.
3. **Traceable**: Preserve the original cause (source error).

## 2. The `CueError` Enum

Defined in `crates/cue_common/src/errors.rs`.

```rust
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum CueError {
    #[error("Workspace not initialized")]
    #[diagnostic(code(cuedeck::workspace::missing), help("Run `cue init` to create a new workspace."))]
    WorkspaceMissing,

    #[error("File not found: {path}")]
    #[diagnostic(code(cuedeck::io::not_found))]
    FileNotFound {
        path: String,
        #[help]
        suggestion: Option<String>,
    },

    #[error("Circular Dependency Detected")]
    #[diagnostic(code(cuedeck::dag::cycle), help("Remove result reference in {cycle_path}"))]
    CircularDependency { cycle_path: String },

    #[error("Token Limit Exceeded")]
    #[diagnostic(code(cuedeck::llm::token_limit))]
    TokenLimitExceeded { current: usize, limit: usize },

    #[error(transparent)]
    Io(#[from] std::io::Error),
    
    #[error(transparent)]
    Config(#[from] config::ConfigError),
}
```

## 3. Mapping Strategy

### 3.1 From 3rd Party to CueError

Use `thiserror`'s `#[from]` attribute for automatic conversion where possible. For context-aware errors, use `map_err`:

```rust
// Example: Wrapping generic IO error with Path context
pub fn read_file(path: &Path) -> Result<String, CueError> {
    std::fs::read_to_string(path).map_err(|_| CueError::FileNotFound { 
        path: path.display().to_string(), 
        suggestion: None 
    })
}
```

### 3.2 Reporting to CLI (`miette`)

In `main.rs`, return `miette::Result<()>`. This validates strictly typed errors into beautiful output.

```text
Error:   × File not found: docs/missing.md
  help: Did you mean 'docs/existing.md'?
```

### 3.3 Reporting to MCP (JSON-RPC)

When running as `cue_mcp`, errors must be serialized to JSON.

```rust
impl From<CueError> for JsonRpcError {
    fn from(err: CueError) -> Self {
        let code = match err {
            CueError::FileNotFound { .. } => 1001,
            CueError::CircularDependency { .. } => 1002,
            _ => -32603, // Internal Error
        };
        JsonRpcError { code, message: err.to_string(), data: None }
    }
}
```

---
**Related Docs**: [MODULE_DESIGN.md](./MODULE_DESIGN.md), [TOOLS_SPEC.md](../04_tools_and_data/TOOLS_SPEC.md)
