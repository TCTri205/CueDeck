# Testing Strategy

## 1. Unit Testing (Rust)

- **Framework**: `cargo test`
- **Property-Based Testing**: Use `proptest` for the Parser interactions.
  - *Scenario*: Generate random Markdown strings -> Ensure Parser never panics.
  - *Scenario*: Generate Circular Graphs -> Ensure DAG resolver always returns Error.

### Property-Based Testing Examples

```rust
use proptest::prelude::*;

proptest! {
    /// Parser should never panic on arbitrary input
    #[test]
    fn parser_never_panics(input in ".*") {
        let _ = Parser::parse_content(&input); // No panic = pass
    }
    
    /// Cycle detection always catches cycles
    #[test]
    fn cycle_detection_works(edges in prop::collection::vec((0..10usize, 0..10usize), 1..20)) {
        let graph = build_graph_from_edges(&edges);
        if has_cycle(&edges) {
            prop_assert!(matches!(graph.resolve(), Err(CueError::CircularDependency { .. })));
        }
    }
    
    /// Token count is always non-negative
    #[test]
    fn token_count_valid(content in ".*") {
        let tokens = tiktoken::count_tokens(&content);
        prop_assert!(tokens >= 0);
    }
}
```

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
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, nightly]
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@${{ matrix.rust }}
      
      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Unit Tests
        run: cargo test --workspace --all-features
        
      - name: Snapshot Tests (Insta)
        run: |
          cargo install cargo-insta
          cargo insta test --unreferenced=reject --check
        # FAIL: If snapshots don't match or unreferenced snapshots exist
      
      - name: Code Coverage
        if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --all-features --engine llvm
      
      - name: Upload coverage
        if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
        uses: codecov/codecov-action@v3
        with:
          fail_ci_if_error: true  # FAIL: If coverage upload fails
          
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Format Check
        run: cargo fmt --all -- --check
        # FAIL: If code is not formatted
      
      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
        # FAIL: If any clippy warnings exist
      
  validate-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build CLI
        run: cargo build --release
      
      - name: Run Cue Doctor
        working-directory: ./
        run: |
          ./target/release/cue init --force
          ./target/release/cue doctor --verbose
        # FAIL: If cue doctor finds issues
      
      - name: Check markdown links
        uses: gaurav-nelson/github-action-markdown-link-check@v1
        with:
          use-quiet-mode: 'yes'
          config-file: '.github/markdown-link-check.json'
        # FAIL: If broken links in docs
  
  coverage-gate:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - name: Check Coverage Threshold
        run: |
          # Coverage must meet minimum thresholds per module
          # cue_core::parser >= 90%
          # cue_core::graph >= 95%
          # cue_config >= 80%
          # cue_mcp >= 85%
          # cue_cli >= 70%
        # FAIL: If any module below threshold
```

### CI Fail Conditions

| Condition | Action | Exit Code |
| :--- | :--- | :--- |
| **Test failure** | Any `cargo test` failure | Non-zero |
| **Clippy warnings** | Any lints with `-D warnings` | 1 |
| **Format check** | Code not formatted | 1 |
| **Coverage drop** | Module coverage below target | 1 |
| **Snapshot mismatch** | Insta snapshots changed | 1 |
| **Broken doc links** | Dead links in markdown | 1 |
| **Cue doctor fails** | Health check issues | Varies (1-6) |

### Snapshot Retention Policy

```yaml
# .github/workflows/cleanup-snapshots.yml
name: Cleanup Old Snapshots

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday

jobs:
  cleanup:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Remove snapshots older than 30 days
        run: |
          find tests/snapshots -name "*.snap.old" -mtime +30 -delete
          # Keep: .snap (current), .snap.new (pending review)
          # Delete: .snap.old (> 30 days)
```

## 6. Performance Benchmarking

- **Tool**: `criterion`
- **Command**: `cargo bench`
- **Target**: `<5ms` for incremental updates.

## 7. Mocking Strategy

### Trait-Based Mocking

```rust
/// Trait for file system operations (mockable)
pub trait FileSystem {
    fn read_file(&self, path: &Path) -> Result<String, CueError>;
    fn write_file(&self, path: &Path, content: &str) -> Result<(), CueError>;
    fn file_exists(&self, path: &Path) -> bool;
}

/// Real implementation
pub struct RealFs;
impl FileSystem for RealFs { /* use std::fs */ }

/// Mock for testing
pub struct MockFs {
    files: HashMap<PathBuf, String>,
}
impl FileSystem for MockFs { /* use in-memory HashMap */ }
```

### Test Coverage Targets

| Module | Target Coverage | Critical Paths |
| :--- | :--- | :--- |
| `cue_core::parser` | > 90% | Frontmatter, anchor extraction |
| `cue_core::graph` | > 95% | Cycle detection, topological sort |
| `cue_config` | > 80% | Config loading, cascading |
| `cue_mcp` | > 85% | JSON-RPC handling, error mapping |
| `cue_cli` | > 70% | Command dispatch |

## 8. Test Matrices

### MCP Tools Test Matrix

| Tool | Scenario | Input | Expected Output | Error Code |
| :--- | :--- | :--- | :--- | :--- |
| `read_context` | Valid query | `{"query": "auth"}` | Results array (score desc) | - |
| `read_context` | Empty query | `{"query": ""}` | Invalid params error | -32602 |
| `read_context` | Token limit hit | `{"query": "docs", "limit": 1000}` | Token limit error | 1003 |
| `read_doc` | Valid file | `{"path": "docs/api.md"}` | File content + token count | - |
| `read_doc` | File not found | `{"path": "missing.md"}` | File not found error | 1001 |
| `read_doc` | Stale cache | Modify file, don't refresh | Stale cache error | 1006 |
| `read_doc` | With anchor | `{"path": "doc.md", "anchor": "Login"}` | Section content only | - |
| `list_tasks` | Filter by status | `{"status": "active"}` | Filtered tasks array | - |
| `list_tasks` | No filters | `{}` | All tasks | - |
| `update_task` | Valid update | `{"id": "abc123", "updates": {...}}` | Updated task object | - |
| `update_task` | Invalid ID | `{"id": "INVALID"}` | Invalid params error | -32602 |
| `update_task` | Lock conflict | Update while locked | Lock error | 1007 |

### Parser Test Matrix

| File Type | Edge Case | Expected Behavior |
| :--- | :--- | :--- |
| Markdown | Empty file | Document with 0 anchors, minimal tokens |
| Markdown | Only frontmatter | Valid Document, anchors = [] |
| Markdown | Invalid frontmatter YAML | Parsing error with line number |
| Markdown | Nested headings (H1 > H2 > H3) | Anchors sorted by start_line |
| Markdown | Duplicate anchor names | Both anchors present, distinguished by line |
| Markdown | Very long file (> 1MB) | Parses successfully or memory limit |
| Markdown | Unicode characters | Correct slug generation (URL-safe) |
| Markdown | Code blocks with `---` | Not confused with frontmatter |

### Watcher Test Matrix & Flakiness Mitigation

| Event Type | Debounce Scenario | Expected Behavior | Flakiness Mitigation |
| :--- | :--- | :--- | :--- |
| File created | Single new file | One update event | Wait 600ms (debounce + buffer) |
| File modified | Rapid edits (< 500ms apart) | Single batched update | Use `tokio::time::sleep(600ms)` |
| File deleted | File removed | Cache invalidation | Verify via polling, not just event |
| Multiple files | 5 files changed at once | Batched update | Test with serial writes + sleep between |
| Rename | `mv old.md new.md` | Delete + Create events | Handle both event orders |
| Network drive | Remote file change | May be delayed | Increase timeout to 2s for flaky FS |

**Watcher Flakiness Mitigation Strategies**:

1. **Mandatory sleep**: Always `sleep(600ms)` after filesystem operation
2. **Polling verification**: Don't rely on events alone, verify final state via `fs::metadata`
3. **Retry on timeout**: If event not received in 2s, poll cache state
4. **Serial test execution**: Mark watcher tests `#[serial]` to avoid race conditions
5. **Platform-specific timeouts**: Windows needs longer debounce (750ms vs 500ms)

---
**Related Docs**: [EVALUATION_METRICS.md](./EVALUATION_METRICS.md), [TROUBLESHOOTING.md](./TROUBLESHOOTING.md), [CONTRIBUTING.md](../01_general/CONTRIBUTING.md)
