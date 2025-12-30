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

### `Context Compression API`

```rust
/// Compress context to fit within token budget
pub struct ContextCompressor {
    abbreviations: HashMap<String, String>,
    token_counter: TokenCounter,
}

impl ContextCompressor {
    /// Compress with multi-stage pipeline
    /// Achieves ~40% token reduction
    pub fn compress(&self, context: &FullContext, budget: usize) 
        -> Result<CompressedContext, CueError>;
    
    /// Compression stages (in order of safety):
    /// 1. Remove comments (preserve docstrings)
    /// 2. Abbreviate keywords (async function → async fn)
    /// 3. Compress whitespace
    /// 4. Summarize long sections (>30 lines)
    /// 5. Reference by hash (large files)
}

pub struct CompressedContext {
    pub content: String,
    pub tokens_used: usize,
    pub compression_ratio: f32,
}
```

### `Validation Engine API`

```rust
/// Rules-based validation for code changes
pub struct RulesEngine {
    rules: Vec<Rule>,
    rule_cache: HashMap<String, Vec<Rule>>,
}

impl RulesEngine {
    /// Load rules from .cuedeck/security.rules
    pub fn load_rules(&mut self, path: &Path) -> Result<(), CueError>;
    
    /// Validate single file
    pub fn validate_file(&self, path: &Path, content: &str) 
        -> ValidationResult;
    
    /// Validate entire change set before commit
    pub fn validate_changes(&self, changes: &[Change]) 
        -> ChangeValidationResult;
}

pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<Violation>,
    pub warnings: Vec<String>,
}

pub struct Violation {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub line: usize,
    pub suggestion: String,
}

pub enum Severity { Critical, High, Medium, Low }
```

### `Session Management API`

```rust
/// Manage session state for context continuity
pub struct SessionStateManager {
    state: SessionState,
    session_path: PathBuf,
}

impl SessionStateManager {
    /// Create new session for workflow
    pub fn create_session(&mut self, workflow: &str) 
        -> Result<SessionState, CueError>;
    
    /// Load existing session
    pub fn load_session(&mut self, session_id: &str) 
        -> Result<SessionState, CueError>;
    
    /// Update working set (files being modified)
    pub fn update_working_set(&mut self, files: &[PathBuf]);
    
    /// Log architectural decision
    pub fn record_decision(&mut self, 
        title: &str, 
        rationale: &str,
        alternatives: &[&str],
        affects: &[PathBuf]
    );
    
    /// Track assumption about project
    pub fn record_assumption(&mut self,
        assumption: &str,
        source: &str,
        impact: Impact
    );
    
    /// Validate all assumptions before critical decision
    pub fn validate_assumptions(&self) -> ValidateResult;
    
    /// Refresh stale context
    pub fn refresh(&mut self) -> Result<(), CueError>;
    
    /// Generate context summary for handoff
    pub fn generate_context_summary(&self) -> String;
}
```

---
**Related Docs**: [TOOLS_SPEC.md](./TOOLS_SPEC.md), [ERROR_HANDLING_STRATEGY.md](../02_architecture/ERROR_HANDLING_STRATEGY.md), [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md)
