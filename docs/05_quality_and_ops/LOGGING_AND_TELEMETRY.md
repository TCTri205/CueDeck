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

---
**Related Docs**: [TECH_STACK.md](../02_architecture/TECH_STACK.md), [ERROR_HANDLING_STRATEGY.md](../02_architecture/ERROR_HANDLING_STRATEGY.md)
