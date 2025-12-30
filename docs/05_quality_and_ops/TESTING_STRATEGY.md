# Testing Strategy

## 1. Unit Testing (Rust)

- **Framework**: `cargo test`
- **Property-Based Testing**: Use `proptest` for the Parser interactions.
  - *Scenario*: Generate random Markdown strings -> Ensure Parser never panics.
  - *Scenario*: Generate Circular Graphs -> Ensure DAG resolver always returns Error.

## 2. Integration Testing

- **Snapshot Testing**: Use `insta` to verify CLI outputs.
  - *Command*: `cargo install cargo-insta`.
  - *Test*: Run `cue scene`, capture output. Compare with `tests/snapshots/reference_scene.snap`.
- **E2E Trace**:
    1. Create temp workspace.
    2. `cue init`.
    3. `cue card new "test"`.
    4. Assert file existence and JSON Frontmatter validity.

## 3. MCP Protocol Testing

- **Mock Client**: Use `python/test_mcp_client.py`.
  - Sends: `{"method": "read_context", ...}` to `stdin`.
  - Asserts: `stdout` contains valid JSON-RPC Response.
  - Asserts: `stderr` contains proper logs (not leaked to stdout).

  - Asserts: `stderr` contains proper logs (not leaked to stdout).

## 4. File Watcher Testing (Async)

Testing the watcher involves filesystem race conditions. Use this pattern:

### Strategy: `tempfile` + `tokio::time::sleep`

1. **Setup**: Create a `tempfile::tempdir()`.
2. **Spawn**: Start `cue watch` in a background `tokio::task`.
3. **Action**: Write a file to the temp dir.
4. **Wait**: Sleep 100ms (debounce window is 500ms, so sleep 600ms).
5. **Assert**: specific side-effect (e.g., Clipboard updated, or a "sentinel file" created).

```rust
#[tokio::test]
async fn test_watcher_detects_change() {
    let dir = tempfile::tempdir().unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    
    // Inject channel into watcher for testing
    let _handle = tokio::spawn(async move {
        run_watcher_with_callback(dir.path(), || tx.send(()).await).await;
    });

    // Write file
    std::fs::write(dir.path().join("test.md"), "content").unwrap();
    
    // Assert event received within 1s
    assert!(tokio::time::timeout(Duration::from_secs(1), rx.recv()).await.is_ok());
}
```

## 5. CI/CD Pipeline (GitHub Actions)

**File**: `.github/workflows/ci.yml`

```yaml
name: CueDeck CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Unit Tests
        run: cargo test --workspace

      - name: Snapshot Tests (Insta)
        run: cargo install cargo-insta && cargo insta test --unreferenced=reject

  validate-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # Assuming we bootstap cue binary here
      - name: Run Cue Doctor
        run: ./target/release/cue doctor
```

## 5. Performance Benchmarking

- **Tool**: `criterion`
- **Command**: `cargo bench`
- **Target**: `<5ms` for incremental updates.

---
**Related Docs**: [EVALUATION_METRICS.md](./EVALUATION_METRICS.md), [TROUBLESHOOTING.md](./TROUBLESHOOTING.md), [CONTRIBUTING.md](../01_general/CONTRIBUTING.md)
