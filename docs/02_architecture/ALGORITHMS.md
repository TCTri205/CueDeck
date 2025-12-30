# Algorithm Specifications

## 1. Context Resolution DAG (Directed Acyclic Graph)

The core of CueDeck is the ability to interpret a list of files as a coherent "Scene".

### 1.1 Cycle Detection (DFS)

We use a standard Depth-First Search with a `recursion_stack` to detect cycles.

```rust
fn has_cycle(node, visited, stack) -> bool {
    visited.add(node);
    stack.add(node);
    
    for child in graph.get(node) {
        if !visited.contains(child) {
             if has_cycle(child, visited, stack) return true;
        } else if stack.contains(child) {
             return true; // CYCLE DETECTED
        }
    }
    stack.remove(node);
    return false;
}
```

**Policy**: If a cycle is detected, the `cue scene` command FAILS immediately with Error `1002`, listing the cycle path (`A -> B -> A`).

### 1.2 Topological Sort (Linearization)

Once verified acyclic, we flatten the graph to produce the `SCENE.md` order.

- **Priority**:
    1. Active Card (Root).
    2. Direct Children (Depth 1).
    3. Indirect Children (Depth 2+).
    4. Sibling/Global Docs (if space permits).

## 2. Token Pruning (The "Knapsack-Lite")

We do not solve the full Knapsack problem; we use a Greedy approach.

- **Budget**: `N` tokens (default 32k).
- **Algorithm**:
    1. Start with `Empty Buffer`.
    2. Add **Active Card** (Critical). Update `UsedTokens`.
    3. Iterate through **Topological Sorted List**:
        - `Estimate` token count of Node `i`.
        - If `UsedTokens + Estimate(i) < N`:
            - Append Node `i` content.
            - `UsedTokens += Estimate(i)`.
        - Else:
            - Append `> [!WARNING] Context Truncated here...`
            - Break.

## 3. Anchor Extraction

How we identify specific sections: `@doc/api#Login`.

1. **Parse**: `pulldown-cmark` event stream.
2. **State Machine**:
    - `Scanning`: Looking for Header Event with text matching "Login".
    - `Capturing`: Found Header. Record events.
        - **Depth Check**: Store current header level (e.g., H2).
    - `Stopping`:
        - Next Event is Header <= Saved Level (H1 or H2).
        - Or End-of-File.

---
**Related Docs**: [SYSTEM_ARCHITECTURE.md](./SYSTEM_ARCHITECTURE.md), [MODULE_DESIGN.md](./MODULE_DESIGN.md), [GLOSSARY.md](../01_general/GLOSSARY.md)
