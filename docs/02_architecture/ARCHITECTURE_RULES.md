# Architecture Rules

This document defines the architecture rules and constraints for the CueDeck project.

## 1. Rust Workspace Architecture

CueDeck follows a modular Rust Workspace architecture:

```text
cuedeck-workspace/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── cue_common/         # Shared types, errors, constants
│   ├── cue_config/         # Configuration loading/merging
│   ├── cue_core/           # Business logic (parser, cache, DAG)
│   ├── cue_cli/            # User-facing CLI/TUI
│   └── cue_mcp/            # MCP Server (JSON-RPC)
└── tests/                  # Integration tests
```

**Dependency Direction**: `cue_cli` → `cue_core` → `cue_config` → `cue_common`

**Why:** Separation of concerns, independent versioning, faster incremental builds.

## 2. Crate Dependency Rules

### Allowed Dependencies

```text
cue_cli  → cue_core, cue_config, cue_common
cue_mcp  → cue_core, cue_config, cue_common
cue_core → cue_config, cue_common
cue_config → cue_common
cue_common → (external crates only)

Data flows DOWN: CLI/MCP → Core → Config → Common
```

### Forbidden Patterns

- ❌ Circular crate dependencies
- ❌ `cue_common` importing from other cue_* crates
- ❌ Direct file I/O in `cue_common` (types only)

## 3. Type Safety Requirements

**REQUIRED:**

- All functions MUST have explicit return types
- All parameters MUST be typed
- Avoid `dyn Any` and type erasure unless justified

```rust
// ✅ Good
fn get_user_by_id(id: &str) -> Result<User, CueError> {
    // ...
}

// ❌ Bad - no return type
fn get_user_by_id(id: &str) {
    // ...
}
```

## 4. Error Handling

**REQUIRED:**

- All async functions must have error handling
- Custom error types for domain errors
- Errors must be propagated with context

```rust
// ✅ Good
pub enum CueError {
    NotFound(String),
    Parse(String),
    Io(std::io::Error),
}
```

## 5. Testing Requirements

**MANDATORY:**

- All business logic (services) must have tests
- Minimum 80% code coverage for critical paths
- Unit + integration tests required

```text
tests/
├── snapshot_scene.rs        # Output verification
├── watcher_integration.rs   # Async flows
└── mcp_protocol.rs          # JSON-RPC tests
```

## 6. Configuration Management

**RULE:** No hardcoded values in code.

```rust
// ❌ Bad
const API_TIMEOUT: u64 = 5000; // hardcoded

// ✅ Good
let api_timeout = config.api.timeout; // from config
```

## 7. Dependency Management

**RULES:**

- No circular dependencies
- Dependencies must form a DAG (directed acyclic graph)
- Use dependency injection for testability

## 8. Performance Constraints

| Metric | Target | Failure Threshold |
| :--- | :--- | :--- |
| **CLI responses** | < 200ms (p95) | > 500ms |
| **File parsing** | < 100ms (p95) | > 300ms |
| **Memory usage** | < 500MB | > 1GB |
| **Cache hit rate** | > 95% | < 80% |

## 9. Security Requirements

**MANDATORY:**

- Input validation on all MCP tool inputs
- Path traversal prevention (reject `..` in paths)
- Secret masking before any output
- No `unsafe` blocks without explicit justification
- Sandbox file access to workspace directory only

## 10. Logging Requirements

**REQUIRED:**

- All critical operations logged
- Structured logging (JSON format)
- Error tracking with context

```rust
// ✅ Good - structured logging
tracing::info!(
    user_id = %user.id,
    action = "user_created",
    "User created successfully"
);
```

---
**Related Docs**: [SYSTEM_ARCHITECTURE.md](./SYSTEM_ARCHITECTURE.md), [MODULE_DESIGN.md](./MODULE_DESIGN.md), [SECURITY.md](./SECURITY.md)
