# Engineering Standards

> [!IMPORTANT] **Enforce Consistency**
>
> These rules prevent "hallucinated decisions" where agents invent their own patterns.
> All technical decisions MUST follow these standards unless explicitly overridden by architecture.

## Error Handling

### Format: Always `miette::Result<T>`

```rust
use miette::Result;

pub fn parse_task(content: &str) -> Result<Task> {
    // Implementation
}
```

**Forbidden**:

- `std::result::Result<T, E>` in public APIs (use miette wrapper)
- Panic in libraries (`unwrap()`, `expect()`)
- Silent error swallowing (`.ok()` without logging)

### Error Context

ALWAYS add context with `.wrap_err()`:

```rust
use miette::IntoDiagnostic;

Connection::open(path)
    .into_diagnostic()
    .wrap_err_with(|| format!("Failed to open database at {}", path))?;
```

### Error Taxonomy

```rust
use thiserror::Error;
use miette::Diagnostic;

#[derive(Error, Diagnostic, Debug)]
pub enum CueError {
    #[error("Failed to parse task frontmatter")]
    #[diagnostic(code(cue::parse::frontmatter))]
    FrontmatterParse {
        #[source_code]
        src: String,
        #[label("invalid YAML here")]
        span: miette::SourceSpan,
    },
    
    #[error("Circular dependency detected")]
    #[diagnostic(code(cue::graph::cycle))]
    CircularDependency {
        cycle: Vec<String>,
    },
}
```

---

## Retry & Timeout

### HTTP Requests

```toml
[http]
max_retries = 3
base_delay_ms = 100  # Exponential backoff: 100ms, 200ms, 400ms
timeout_ms = 5000
```

**Implementation**:

```rust
use tokio::time::{timeout, sleep, Duration};

async fn fetch_with_retry(url: &str) -> Result<String> {
    let mut delay = Duration::from_millis(100);
    
    for attempt in 1..=3 {
        match timeout(Duration::from_millis(5000), reqwest::get(url)).await {
            Ok(Ok(response)) => return Ok(response.text().await?),
            Ok(Err(e)) => {
                tracing::warn!("HTTP request failed", attempt, error = %e);
                if attempt < 3 {
                    sleep(delay).await;
                    delay *= 2;  // Exponential backoff
                }
            }
            Err(_) => {
                tracing::warn!("HTTP request timeout", attempt);
                if attempt < 3 {
                    sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }
    
    Err(CueError::HttpTimeout)
}
```

### Database Operations

```toml
[database]
busy_timeout_ms = 5000
max_retries = 5
retry_delay_ms = 50
```

```rust
use rusqlite::Connection;

let mut conn = Connection::open(db_path)?;
conn.busy_timeout(std::time::Duration::from_millis(5000))?;
```

---

## Logging Levels

| Level   | Use Case             | Example                                        |
|:--------|:---------------------|:-----------------------------------------------|
| `error` | Unrecoverable errors | Database corruption, missing critical files    |
| `warn`  | Recoverable issues   | Cache miss, retry attempt, deprecated API usage |
| `info`  | Business events      | Task created, scene generated, search completed |
| `debug` | Development details  | Function entry/exit, config values             |
| `trace` | Very verbose         | Loop iterations, internal state changes        |

### Examples

```rust
use tracing::{error, warn, info, debug, trace};

// ✅ CORRECT
info!("Task created", task_id = %task.id, title = %task.title);
debug!("Cache hit", key = %cache_key, age_ms = elapsed);
trace!("Parsing line", line_num = i, content = %line);

// ❌ FORBIDDEN - Sensitive data
error!("Auth failed: {:?}", user);  // Contains password hash
debug!("Request: {:?}", http_req);  // Contains Authorization header
```

### Structured Logging

```rust
use tracing::instrument;

#[instrument(skip(conn), fields(task_id = %id))]
pub async fn get_task(conn: &Connection, id: &str) -> Result<Task> {
    // Automatically logs function entry/exit with task_id
}
```

---

## Pagination

### Standard: Cursor-based

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub total: Option<usize>,  // Optional for performance
}

pub async fn list_tasks(
    cursor: Option<&str>,
    limit: usize,
) -> Result<PaginatedResult<Task>> {
    let limit = limit.min(100);  // Enforce max
    
    // Implementation
}
```

**Limits**:

- Default page size: `50`
- Max page size: `100`
- Cursor format: Base64-encoded timestamp or ID

**Forbidden**:

- Offset-based pagination (slow for large datasets)
- Unbounded queries (must have limit)

---

## Caching Strategy

### Cache Invalidation

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct Cache<K, V> {
    inner: HashMap<K, CachedValue<V>>,
}

struct CachedValue<V> {
    value: V,
    created_at: Instant,
}

impl<K: Eq + std::hash::Hash, V> Cache<K, V> {
    pub fn set(&mut self, key: K, value: V) {
        self.inner.insert(key, CachedValue {
            value,
            created_at: Instant::now(),
        });
    }
    
    pub fn get(&self, key: &K, max_age: Duration) -> Option<&V> {
        self.inner.get(key).and_then(|cached| {
            if cached.created_at.elapsed() < max_age {
                Some(&cached.value)
            } else {
                None  // Expired
            }
        })
    }
    
    pub fn invalidate(&mut self, key: &K) {
        self.inner.remove(key);
    }
}
```

### TTL Defaults

```toml
[cache.ttl]
file_metadata = "5m"
search_results = "1m"
scene_output = "30s"
graph_resolution = "10m"
```

**Invalidate on**:

- File writes (invalidate file metadata)
- Task creation/update (invalidate search, graph)
- Configuration change (invalidate everything)

---

## Naming Conventions

### Functions

- **Query (read-only)**: `get_*`, `find_*`, `list_*`, `search_*`
- **Command (side effects)**: `create_*`, `update_*`, `delete_*`, `execute_*`
- **Bool check**: `is_*`, `has_*`, `can_*`, `should_*`
- **Conversion**: `to_*`, `from_*`, `into_*`, `as_*`

```rust
// ✅ CORRECT
pub fn get_task(id: &str) -> Result<Task>;        // Query
pub fn create_task(title: &str) -> Result<Task>;  // Command
pub fn is_valid(task: &Task) -> bool;             // Bool check
pub fn to_json(task: &Task) -> String;            // Conversion

// ❌ INCORRECT
pub fn task(id: &str) -> Result<Task>;            // Ambiguous
pub fn new_task(title: &str) -> Result<Task>;     // Use create_*
pub fn valid(task: &Task) -> bool;                // Use is_valid
```

### Types

- **Struct**: `PascalCase` (e.g., `TaskMetadata`, `SearchResult`)
- **Enum**: `PascalCase` variants (e.g., `TaskStatus::Active`)
- **Error**: `*Error` suffix (e.g., `ParseError`, `CueError`)
- **Config**: `*Config` suffix (e.g., `DatabaseConfig`)

### Modules

- **snake_case**: `cue_core`, `task_parser`, `graph_resolver`
- Singular vs plural: Use singular for type modules, plural for collections
  - `task.rs` (defines `Task` type)
  - `tasks.rs` (operations on multiple tasks)

---

## Async Patterns

### ALWAYS use `tokio::spawn` for independent work

```rust
use tokio::task;

let handle = task::spawn(async move {
    // Independent computation (e.g., background cache refresh)
    expensive_computation().await
});

// Do other work...

let result = handle.await??;
```

### Cancellation Safety

```rust
use tokio::select;

select! {
    result = operation() => {
        // Handle result
    }
    _ = cancellation_token.cancelled() => {
        // Cleanup on cancellation
        return Err(CueError::Cancelled);
    }
}
```

### NEVER

- Block async tasks with `std::thread::sleep` (use `tokio::time::sleep`)
- Use `.unwrap()` in `async fn` (propagate errors with `?`)
- Spawn blocking operations in async context without `spawn_blocking`

```rust
// ❌ FORBIDDEN
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));  // Blocks executor
    heavy_cpu_work();  // Blocks other tasks
}

// ✅ APPROVED
async fn good() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    tokio::task::spawn_blocking(|| heavy_cpu_work()).await?;
}
```

---

## Configuration

### Cascading Order

1. Default values (in code)
2. Global config (`~/.config/cuedeck/config.toml`)
3. Workspace config (`.cuedeck/config.toml`)
4. Environment variables (`CUE_*`)
5. CLI flags (`--flag`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,
}

fn default_cache_ttl() -> u64 {
    300  // 5 minutes
}
```

### Environment Variables

- Prefix: `CUE_`
- Format: `CUE_SECTION_KEY` (e.g., `CUE_DATABASE_PATH`)
- Boolean: `true`/`false` or `1`/`0`

---

## Testing

### Unit Test Naming

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_frontmatter() {
        // Arrange
        let input = "---\nstatus: active\n---\n";
        
        // Act
        let result = parse_frontmatter(input).unwrap();
        
        // Assert
        assert_eq!(result.status, TaskStatus::Active);
    }
    
    #[test]
    fn test_parse_empty_frontmatter_returns_error() {
        let input = "";
        assert!(parse_frontmatter(input).is_err());
    }
}
```

### Property-based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_roundtrip_serialization(task_id in "[a-z0-9]{6}") {
        let task = Task::new(&task_id);
        let json = serde_json::to_string(&task)?;
        let parsed: Task = serde_json::from_str(&json)?;
        assert_eq!(task.id, parsed.id);
    }
}
```

---

## Performance

### Benchmarking

Use `criterion` for benchmarks:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parse(c: &mut Criterion) {
    let input = std::fs::read_to_string("fixtures/large.md").unwrap();
    
    c.bench_function("parse_large_file", |b| {
        b.iter(|| parse_task(black_box(&input)))
    });
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
```

### Optimization Guidelines

- Profile before optimizing (use `cargo flamegraph`)
- Avoid premature optimization
- Clone only when necessary (use references)
- Use `Cow<str>` for conditional ownership
- Prefer iterators over collecting into `Vec`

---

## Decision Framework

When agent encounters undefined behavior:

```text
📋 TECHNICAL DECISION NEEDED

Context: Implementing task filtering
Question: Should we use SQL WHERE clause or filter in Rust?

Options:
1. SQL WHERE (faster for large datasets)
2. Rust filter (more flexible, easier to test)

Recommendation: Option 1 (expected dataset: 1000+ tasks)
Rationale: Performance > flexibility for this use case

Please confirm or suggest alternative.
```

Agent MUST NOT:

- Invent new patterns without asking
- Choose arbitrarily between equal options
- Assume performance characteristics

---

**Related Docs**:

- [APPROVED_LIBRARIES.md](./APPROVED_LIBRARIES.md)
- [RUST_CODING_STANDARDS.md](../05_quality_and_ops/RUST_CODING_STANDARDS.md)
- [SECURITY_RULES.md](../04_security/SECURITY_RULES.md)
- [ARCHITECTURE.md](../02_architecture/ARCHITECTURE.md)
