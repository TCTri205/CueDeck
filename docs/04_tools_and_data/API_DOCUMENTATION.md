# API Documentation

## 1. MCP Interface (JSON-RPC over Stdio)

Protocol compliance: **MCP 2024-11 Draft** + **JSON-RPC 2.0**.

### Request Lifecycle

```json
Request (Stdin) -> Deserializer -> Dispatcher -> Async Handler -> Serializer -> Response (Stdout)
```

### Error Codes

| Code | Message | Description |
| :--- | :--- | :--- |
| `-32700` | Parse Error | Invalid JSON received on stdin. |
| `-32601` | Method Not Found | Tool name typo or version mismatch. |
| `1001` | File Not Found | Path exists in Cache but generic IO failed (Race Condition). |
| `1002` | Cycle Detected | Graph resolution found `A -> B -> A`. |
| `1003` | Token Limit Exceeded | Scene is too large even after aggressive pruning. |

## 2. Rust Internal API (`crates/cue_core`)

### `Public Types`

```rust
// The Atomic Unit of Knowledge
pub struct Document {
    pub path: PathBuf,
    pub hash: String, // SHA256
    pub anchors: Vec<Anchor>,
    pub frontmatter: Option<serde_yaml::Value>,
}

// A Specific Target in the Graph
pub struct Anchor {
    pub header: String, // "API > Login"
    pub level: u8,      // 1-6
    pub start_line: usize,
    pub end_line: usize,
}
```

### `Engine API`

- `Parser::parse_file(path: &Path) -> Result<Document, CueError>`
  - Handles reading, hashing, caching logic transparently.
- `Graph::resolve(root: &Document) -> Dag<Document>`
  - Returns a linearized list of documents suitable for concatenation.

---
**Related Docs**: [TOOLS_SPEC.md](./TOOLS_SPEC.md), [ERROR_HANDLING_STRATEGY.md](../02_architecture/ERROR_HANDLING_STRATEGY.md), [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md)
