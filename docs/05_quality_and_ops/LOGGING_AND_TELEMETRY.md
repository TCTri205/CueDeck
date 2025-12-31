# Logging & Telemetry Strategy

## 1. Philosophy: "Zero Stdio Pollution"

Because CueDeck runs as an MCP Server over `stdin`/`stdout`, **absolutely no logs** can be printed to stdout. Doing so corrupts the JSON-RPC protocol.

- **Stdout**: Pure JSON-RPC responses ONLY.
- **Stderr**: Human-readable logs (for `cue watch` or debug).
- **File**: Persistent logs (`.cuedeck/logs/`).

## 2. Implementation (`tracing`)

We use `tracing-subscriber` with `EnvFilter`.

### Setup

```rust
// crates/cue_common/src/telemetry.rs

pub fn init_tracing(verbose: bool) {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr) // CRITICAL
        .with_target(false)
        .compact();

    let filter = if verbose { 
        "cuedeck=debug" 
    } else { 
        "cuedeck=info" 
    };

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::new(filter))
        .init();
}
```

## 3. Log Levels

| Level | Use Case | Example |
| :--- | :--- | :--- |
| `ERROR` | Operation failed. User action required. | `[ERR] Cycle detected: A->B->A` |
| `WARN` | Degraded state, but proceeding. | `[WARN] Token limit exceeded, pruning...` |
| `INFO` | High-level lifecycle events. | `Scene generated (150ms)` |
| `DEBUG` | Developer context. | `Parsed frontmatter for doc/auth.md` |
| `TRACE` | Noisy loops. | `Checking hash for file X...` |

## 4. Telemetry File

When running in `server` mode, we also write to `.cuedeck/logs/mcp.log` (rotating).

### Log Rotation Configuration

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

/// Create rotating file appender
pub fn create_file_logger() -> RollingFileAppender {
    RollingFileAppender::builder()
        .rotation(Rotation::DAILY)           // New file each day
        .filename_prefix("mcp")              // mcp.2024-01-15.log
        .filename_suffix("log")
        .max_log_files(7)                    // Keep 7 days
        .build(".cuedeck/logs")
        .expect("Failed to create log appender")
}
```

### Structured Log Format

For machine-parseable logs:

```json
{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","target":"cue_core::parser","message":"Parsed file","fields":{"path":"docs/api.md","tokens":1420,"duration_ms":5}}
```

## 5. OpenTelemetry Integration (Optional)

For distributed tracing in production:

```rust
use tracing_opentelemetry::OpenTelemetryLayer;
use opentelemetry::sdk::trace::Tracer;

pub fn init_otel(tracer: Tracer) {
    let otel_layer = OpenTelemetryLayer::new(tracer);
    
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(otel_layer)
        .init();
}
```

### Span Example

```rust
#[tracing::instrument(skip(content), fields(path = %path.display()))]
pub fn parse_file(path: &Path, content: &str) -> Result<Document, CueError> {
    tracing::info!("Starting parse");
    // ... parsing logic ...
    tracing::info!(tokens = result.tokens, "Parse complete");
    Ok(result)
}
```

## 6. Log Retention & Cleanup Policy

### Retention Rules

| Log Type | Retention Period | Rotation | Cleanup Trigger |
| :--- | :--- | :--- | :--- |
| **MCP Server Logs** | 7 days | Daily | Auto (midnight) |
| **Error Logs** | 30 days | Weekly | Manual via `cue clean --logs` |
| **Debug Traces** | 24 hours | Hourly | Auto (on restart) |
| **Audit Logs** | 90 days | Monthly | Compliance policy |
| **Performance Metrics** | 14 days | Daily | Rolling window |

### Cleanup Commands

```bash
# Remove old logs (7+ days)
cue clean --logs

# Remove all logs (fresh start)
cue clean --logs --all

# Compress and archive logs
cue logs archive --since=30d --output=logs-backup.tar.gz
```

### Disk Space Management

```rust
// Auto-cleanup when logs exceed threshold
pub fn check_log_size() -> Result<(), CueError> {
    let log_dir = Path::new(".cuedeck/logs");
    let total_size = calculate_dir_size(log_dir)?;
    
    if total_size > MAX_LOG_SIZE_MB * 1024 * 1024 {
        tracing::warn!("Log size {}MB exceeds limit {}MB, cleaning up", 
                       total_size / (1024*1024), MAX_LOG_SIZE_MB);
        cleanup_old_logs(log_dir, 7)?; // Keep last 7 days
    }
    Ok(())
}
```

## 7. Alerting & Monitoring Rules

### Alert Conditions

| Condition | Severity | Action | Notification |
| :--- | :--- | :--- | :--- |
| `ERROR` log rate >10/min | CRITICAL | Write to stderr + file | Email on-call |
| Cache rebuild >500ms | WARNING | Log performance metric | Dashboard alert |
| Cycle detected | ERROR | Block operation + log | User notification |
| Disk space <100MB | WARNING | Trigger cleanup | CLI warning |
| File watcher dropped events | ERROR | Full rescan | Log + metric |

### Metrics Collection

```rust
use metrics::{counter, histogram};

#[tracing::instrument]
pub fn parse_file(path: &Path) -> Result<Document> {
    let start = Instant::now();
    
    counter!("cuedeck.parser.invocations").increment(1);
    
    let result = do_parse(path)?;
    
    let duration = start.elapsed().as_millis() as f64;
    histogram!("cuedeck.parser.duration_ms").record(duration);
    
    Ok(result)
}
```

### Health Check Endpoint

```bash
# Query system health
cue doctor --format=json

# Expected output:
{
  "status": "healthy",
  "cache_size_mb": 12.5,
  "log_size_mb": 3.2,
  "last_gc": "2024-01-15T10:00:00Z",
  "active_cards": 8,
  "warnings": []
}
```

## 8. PII (Personally Identifiable Information) Protection Rules

### Redaction Policy

All logs MUST be sanitized before being written to prevent leaking sensitive data:

| Data Type | Pattern | Redaction Example | Detection Regex |
| :-------- | :------ | :---------------- | :-------------- |
| **Email Addresses** | <user@example.com> | `***@***.***` | `\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z\|a-z]{2,}\b` |
| **File Paths (Home)** | `/home/john/project` | `/home/***` | `(/home/\|C:\\Users\\)[^/\\s]+` |
| **API Keys** | `sk-abc123...` | `sk-***` | `(sk\|pk)-[a-zA-Z0-9]{20,}` |
| **IP Addresses** | `192.168.1.100` | `***.***.***.***` | `\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b` |
| **Usernames** | `@johndoe` | `@***` | Config-based whitelist |

### Implementation

```rust
// crates/cue_common/src/sanitizer.rs

use regex::Regex;

pub struct LogSanitizer {
    patterns: Vec<(Regex, String)>,
}

impl LogSanitizer {
    pub fn new() -> Self {
        let patterns = vec![
            (Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(), 
             "***@***.***".to_string()),
            (Regex::new(r"(/home/|C:\\Users\\)[^/\s]+").unwrap(), 
             "$1***".to_string()),
            (Regex::new(r"(sk|pk)-[a-zA-Z0-9]{20,}").unwrap(), 
             "$1-***".to_string()),
        ];
        Self { patterns }
    }
    
    pub fn sanitize(&self, message: &str) -> String {
        let mut result = message.to_string();
        for (pattern, replacement) in &self.patterns {
            result = pattern.replace_all(&result, replacement).to_string();
        }
        result
    }
}
```

### Usage in Logs

```rust
#[tracing::instrument]
pub fn process_request(path: &Path) {
    let sanitizer = LogSanitizer::new();
    let safe_path = sanitizer.sanitize(&path.to_string_lossy());
    tracing::info!(path = %safe_path, "Processing request");
}
```

### Configuration Override

Allow users to customize PII patterns via config:

```toml
# .cuedeck/config.toml
[logging.pii]
redact_emails = true
redact_paths = true
redact_ips = false  # Disable IP redaction for internal networks

# Custom patterns
custom_patterns = [
    { regex = "AUTH_TOKEN_[A-Z0-9]+", replacement = "AUTH_TOKEN_***" }
]
```

### Audit Compliance

For teams requiring audit trails, logs can be written in two modes:

1. **Development Mode**: Full verbosity, PII included (local only, never in CI)
2. **Production Mode**: Sanitized, no PII (required for shared logs)

```bash
# Development (local debug)
CUEDECK_LOG_MODE=dev cue scene

# Production (sanitized)
CUEDECK_LOG_MODE=prod cue mcp
```

---
**Related Docs**: [TECH_STACK.md](../02_architecture/TECH_STACK.md), [ERROR_HANDLING_STRATEGY.md](../02_architecture/ERROR_HANDLING_STRATEGY.md), [MAINTENANCE_GUIDE.md](./MAINTENANCE_GUIDE.md)
