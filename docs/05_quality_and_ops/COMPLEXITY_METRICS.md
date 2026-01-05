# Complexity Metrics

> [!IMPORTANT] **Maintainability Guarantee**
>
> These metrics prevent code from becoming too complex to understand and maintain.
> Violations MUST be fixed before merging.

## Overview

CueDeck enforces 2 complexity metrics via Clippy configuration:

| Metric | Threshold | Rationale |
| :----- | :-------- | :-------- |
| Cognitive Complexity | 25 | Maximum mental effort to understand code (unified metric) |
| Function Length | 100 lines | Maximum lines per function |

> [!NOTE]
> **Modern Clippy Change**: As of recent Clippy versions, the `cyclomatic-complexity` lint has been renamed to `cognitive-complexity`. The unified `cognitive-complexity-threshold` now controls both cyclomatic and cognitive complexity analysis.

**Configuration**: [`clippy.toml`](../../clippy.toml)

---

## Metric Definitions

### 1. Cyclomatic Complexity

**Definition**: Number of independent paths through code (based on control flow).

**Formula**: `E - N + 2P`, where:

- `E` = edges in control flow graph
- `N` = nodes
- `P` = connected components

**Triggers**: Each of these adds +1 complexity:

- `if`, `else if`, `match` arms
- `while`, `for`, `loop`
- `&&`, `||` in conditions
- `?` (try operator)

**Example violation**:

```rust
// ❌ Cyclomatic complexity = 25 (exceeds threshold of 20)
pub fn process_task(task: &Task) -> Result<()> {
    if task.status == "active" {
        if task.priority == "high" {
            if task.dependencies.is_empty() {
                // ... 15 more nested conditions
            }
        }
    }
    // ...
}
```

### 2. Cognitive Complexity

**Definition**: Measures **how hard code is to understand** (human perspective).

**Adds complexity**:

- Nested control flow (+1 for each nesting level)
- Recursion (+1)
- `break`, `continue` in loops (+1)
- Sequences of logical operators (`&&`, `||`)

**Does NOT add complexity** (unlike cyclomatic):

- Flat `match` arms without nesting
- Early returns (`if guard { return }`)

**Example violation**:

```rust
// ❌ Cognitive complexity = 28 (exceeds threshold of 25)
pub fn validate_graph(graph: &Graph) -> Result<()> {
    for node in &graph.nodes {           // +1 (loop)
        if !node.is_valid() {             // +2 (nested if)
            for dep in &node.deps {       // +3 (double nested loop)
                if dep.is_circular() {    // +4 (triple nested if)
                    // ... more nesting
                }
            }
        }
    }
    // ...
}
```

### 3. Function Length

**Definition**: Total lines in a function (excluding comments and blank lines).

**Threshold**: 100 lines

**Rationale**:

- Long functions are hard to test
- Difficult to understand in code review
- Usually indicates missing abstractions

**Example violation**:

```rust
// ❌ 150 lines (exceeds threshold of 100)
pub fn generate_scene(context: &Context) -> Result<Scene> {
    // ... 150 lines of logic
}
```

---

## How to Fix Violations

### Strategy 1: Extract Functions

Break complex logic into smaller, named functions:

```rust
// ❌ BEFORE (high complexity)
pub fn process(task: &Task) -> Result<()> {
    if task.status == "active" {
        if task.priority == "high" {
            // 20 lines of logic
        }
    }
    // ...
}

// ✅ AFTER (reduced complexity)
pub fn process(task: &Task) -> Result<()> {
    if task.status == "active" {
        process_active_task(task)?;
    }
    Ok(())
}

fn process_active_task(task: &Task) -> Result<()> {
    if task.priority == "high" {
        process_high_priority(task)?;
    }
    Ok(())
}

fn process_high_priority(task: &Task) -> Result<()> {
    // Extracted logic
    Ok(())
}
```

### Strategy 2: Use Early Returns

Replace nested conditions with guard clauses:

```rust
// ❌ BEFORE (high nesting)
pub fn validate(task: &Task) -> Result<()> {
    if task.title.is_empty() {
        return Err(ValidationError::EmptyTitle);
    } else {
        if task.priority.is_valid() {
            if task.dependencies.len() < 10 {
                // Main logic here
            } else {
                return Err(TooManyDeps);
            }
        } else {
            return Err(InvalidPriority);
        }
    }
}

// ✅ AFTER (flat structure)
pub fn validate(task: &Task) -> Result<()> {
    if task.title.is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    
    if !task.priority.is_valid() {
        return Err(InvalidPriority);
    }
    
    if task.dependencies.len() >= 10 {
        return Err(TooManyDeps);
    }
    
    // Main logic here (no nesting)
    Ok(())
}
```

### Strategy 3: Replace Nested Loops with Iterators

Use iterator chains instead of nested loops:

```rust
// ❌ BEFORE (high complexity)
pub fn find_circular_deps(graph: &Graph) -> Vec<String> {
    let mut circular = Vec::new();
    for node in &graph.nodes {
        for dep in &node.deps {
            if dep.is_circular() {
                circular.push(dep.id.clone());
            }
        }
    }
    circular
}

// ✅ AFTER (lower complexity)
pub fn find_circular_deps(graph: &Graph) -> Vec<String> {
    graph.nodes
        .iter()
        .flat_map(|node| &node.deps)
        .filter(|dep| dep.is_circular())
        .map(|dep| dep.id.clone())
        .collect()
}
```

### Strategy 4: Use Match with Early Return

Replace nested `if` with flat `match`:

```rust
// ❌ BEFORE (high cognitive complexity)
pub fn handle_status(task: &Task) -> Result<()> {
    if task.status == Status::Active {
        // logic
    } else if task.status == Status::Done {
        // logic
    } else if task.status == Status::Blocked {
        // logic
    }
    // ... 10 more conditions
}

// ✅ AFTER (lower cognitive complexity)
pub fn handle_status(task: &Task) -> Result<()> {
    match task.status {
        Status::Active => handle_active(task),
        Status::Done => handle_done(task),
        Status::Blocked => handle_blocked(task),
        // ... other cases
    }
}
```

---

## CI Enforcement

### Local Check

```bash
cargo clippy --all-features --workspace -- -D warnings
```

### CI Pipeline

Enforced automatically in `.github/workflows/ci.yml`:

```yaml
clippy:
  name: Clippy
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    # This step will fail if any complexity thresholds are exceeded
    - run: cargo clippy --all-features --workspace -- -D warnings
```

### Warnings vs Errors

By default, Clippy emits **warnings** for complexity violations. The `-D warnings` flag in CI **promotes warnings to errors**, causing the build to fail.

**Local development**: Warnings only (you can still build)  
**CI/PR**: Errors (blocks merge)

---

## Exceptions

### When to Request Exemption

In rare cases, complexity may be unavoidable:

- State machines with many transitions
- Protocol parsers with extensive pattern matching
- Migration code (temporary, will be removed)

### Process

1. **Try all refactoring strategies first** (see above)
2. If still exceeding threshold, add `#[allow(clippy::...)]` with justification:

```rust
// Approved by [reviewer name] on [date]
// Justification: DAG traversal algorithm requires nested loops
#[allow(clippy::cognitive_complexity)]
pub fn topological_sort(graph: &Graph) -> Result<Vec<Node>> {
    // Complex but algorithmically required logic
}
```

1. **Document in code review** why the complexity is necessary
2. **Add TODO** to refactor in future if possible

---

## Metrics Dashboard

Track codebase-wide complexity trends:

```bash
# Install cargo-geiger for complexity metrics
cargo install cargo-geiger

# Generate complexity report
cargo geiger --all-features
```

**Scheduled CI Job**: `.github/workflows/metrics.yml` runs weekly complexity reports.

---

## Related Documentation

- [ENGINEERING_STANDARDS.md](../03_agent_design/ENGINEERING_STANDARDS.md) - General coding patterns
- [RUST_CODING_STANDARDS.md](./RUST_CODING_STANDARDS.md) - Rust-specific guidelines
- [ci.yml](../../.github/workflows/ci.yml) - CI pipeline that enforces these metrics
- [clippy.toml](../../clippy.toml) - Configuration file with thresholds

---

**Last Updated**: 2026-01-04  
**Version**: 1.0
