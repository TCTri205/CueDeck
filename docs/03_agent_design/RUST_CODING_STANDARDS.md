# Rust Coding Standards

## 1. Async & Concurrency

We use `tokio` as our runtime.

- **Binaries (`main.rs`)**: Use `#[tokio::main]`.
- **Libraries (`lib.rs`)**: Avoid `block_on` inside library code. Make functions `async` and let the caller decide.
- **Spawning**:
  - Prefer `tokio::spawn` for independent background tasks (e.g., File Watcher logic).
  - Use `tokio::select!` for cancellation (e.g., stopping the watcher on Ctrl+C).

## 2. State Management (MCP Server)

The MCP server is stateful (needs to hold the Index/Graph).

- **Pattern**: `Arc<RwLock<Workspace>>`
- **Why RwLock?**: 99% of operations are `read_context` (Read). Only `cue watch` or `update_task` trigger Writes. `RwLock` allows concurrent readers.
- **Avoid**: `Mutex` (unless data is trivial), `static mut`.

```rust
pub struct ServerState {
    pub workspace: Arc<RwLock<Workspace>>,
}
```

## 3. Testing Conventions

- **Unit Tests**: Place inside the same file in a `mod tests` block.
- **Integration Tests**: Place in `tests/` directory (treats crate as a black box).
- **Snapshot Tests**: Use `insta` for large text outputs (like `SCENE.md` generation).

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic() { ... }
}
```

## 4. Comments & Documentation

- **Pub Items**: MUST have `///` doc comments.
- **Complexity**: Explain "Why" not just "What".
- **Links**: Use ``[`StructName`]`` for clickable links in rustdoc.

## 5. Idioms (Clippy)

- We strictly enforce `clippy::pedantic` in CI, but allow specific exclusions in `lib.rs` (`#![allow(clippy::module_name_repetitions)]`).
- **Unwrap**: Allowed ONLY in `tests/` or when checking invariants that verifyably cannot fail. In app code, use `Expect` or propagate `Result`.

---
**Related Docs**: [CONTRIBUTING.md](../01_general/CONTRIBUTING.md), [TESTING_STRATEGY.md](../05_quality_and_ops/TESTING_STRATEGY.md)
