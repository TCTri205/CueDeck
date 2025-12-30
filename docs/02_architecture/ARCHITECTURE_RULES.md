# Architecture Rules

This document defines the architecture rules and constraints for the CueDeck project.

## 1. Layered Architecture

Every project MUST follow this structure:

```text
src/
├── api/              # API routes & controllers
├── services/         # Business logic
├── models/           # Data models & types
├── utils/            # Utility functions
├── middleware/       # HTTP middleware
├── config/           # Configuration
├── tests/            # Test files
└── index.ts          # Entry point
```

**Why:** Separation of concerns, testability, scalability.

## 2. Module Import Rules

### Allowed Dependencies

```text
Level 1: Services import from models
Level 2: Controllers import from services
Level 3: API imports from controllers

Data flows DOWN: api → services → models
```

### Forbidden Patterns

- ❌ Cross-level imports (creates circular dependencies)
- ❌ Circular imports between files
- ❌ Skipping layers (API directly calling database)

## 3. Type Safety Requirements

**REQUIRED:**
- All functions MUST have explicit return types
- All parameters MUST be typed
- No `any` type allowed (unless approved exception)

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
├── unit/
│   ├── services/
│   │   └── *_test.rs
│   └── utils/
├── integration/
│   └── api/
└── e2e/
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
|--------|--------|-------------------|
| **API responses** | < 200ms (p95) | > 500ms |
| **Database queries** | < 100ms (p95) | > 300ms |
| **Memory usage** | < 500MB | > 1GB |
| **N+1 queries** | 0 | Any detected |

## 9. Security Requirements

**MANDATORY:**
- Input validation on all API endpoints
- SQL injection prevention (parameterized queries)
- XSS prevention (sanitize output)
- Rate limiting on endpoints

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
