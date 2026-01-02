# Algorithm Specifications

## Overview

This document specifies the core algorithms powering CueDeck's context management.

```mermaid
graph TB
    A[Active Card] --> B[Parse References]
    B --> C{Cycle Check}
    C -->|Pass| D[Build DAG]
    C -->|Fail| E[Error 1002]
    D --> F[Topological Sort]
    F --> G[Token Pruning]
    G --> H[Secret Masking]
    H --> I[SCENE.md]
    
    style E fill:#f44336,color:#fff
    style I fill:#4CAF50,color:#fff
```

## 1. Context Resolution DAG (Directed Acyclic Graph)

The core of CueDeck is the ability to interpret a list of files as a coherent "Scene".

### 1.1 Cycle Detection (DFS)

We use a standard Depth-First Search with a `recursion_stack` to detect cycles.

**Time Complexity**: O(V + E) where V = files, E = references  
**Space Complexity**: O(V) for visited/stack sets

```mermaid
graph LR
    subgraph "Valid DAG"
        A1[Card A] --> B1[Doc B]
        A1 --> C1[Doc C]
        B1 --> D1[Lib D]
    end
    
    subgraph "Cycle Detected ❌"
        A2[Card A] --> B2[Doc B]
        B2 --> C2[Doc C]
        C2 --> A2
    end
```

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

**Algorithm**: Modified Kahn's algorithm with priority weighting

```mermaid
graph TB
    subgraph "Priority Levels"
        P1["📋 Active Card<br/>Priority: 100"]
        P2["📄 Direct Refs (Depth 1)<br/>Priority: 50"]
        P3["📑 Indirect Refs (Depth 2+)<br/>Priority: 25"]
        P4["🗂️ Global Docs<br/>Priority: 10"]
    end
    
    P1 --> P2 --> P3 --> P4
```

- **Priority**:
    1. Active Card (Root) - Always included first
    2. Direct Children (Depth 1) - High priority
    3. Indirect Children (Depth 2+) - Medium priority
    4. Sibling/Global Docs (if space permits) - Low priority

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

## 4. Security Guard

Runs as a final filter on the Output Buffer. See [SECURITY.md](./SECURITY.md) for implementation details.

## 5. Token Optimization Strategies (Extended)
>
> **Detailed Patterns**: See [`IMPLEMENTATION_PATTERNS.md`](./IMPLEMENTATION_PATTERNS.md)

### Strategy 1: Semantic Compression

```text
Before (850 tokens):
```typescript
export async function fetchUserData(userId: string): Promise<User> {
  try {
    const response = await fetch(`/api/users/${userId}`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const data = await response.json();
    return data as User;
  } catch (error) {
    console.error('Failed to fetch user:', error);
    throw error;
  }
}
```

After (180 tokens):

```text
fn fetchUserData(id: string) -> Promise<User>
  - Fetches from /api/users/{id}
  - Returns User | throws Error
  - See file:complete for impl
```

### Strategy 2: Delta Diffing

```text
Session state:
- File state at start: src/api.ts [hash: abc123]
- Agent changed lines 10-15, 42, 88
- Next agent needs delta, not full file:

Delta format:
src/api.ts:
  L10-15: [new content]
  L42: [new line]
  L88: [modified line]
  [Other 200 lines unchanged - reference as "..."]
```

### Strategy 3: File Importance Scoring

```text
Score = (references * 0.3) + (recent_changes * 0.35) + (in_error_stack * 0.35)

Example ranking for "Add auth to API":
1. src/api.ts (referenced 45 times) - MUST INCLUDE
2. src/auth.ts (changed yesterday) - MUST INCLUDE
3. src/types.ts (imported by api.ts) - INCLUDE IF SPACE
4. src/utils.ts (not related) - SKIP
5. docs/api.md (helpful but not critical) - SKIP
```

### Strategy 4: Smart Caching

```text
Cache levels (in memory, ~50MB max):
L1: Current role + active workflow (always hot)
L2: Last 3 files accessed (1-2s reload)
L3: Project metadata (instant)
L4: Rules & architecture (instant)

Eviction: LRU after 30 min inactivity
```

## 6. Algorithm Complexity Analysis

Comprehensive performance characteristics of all core algorithms:

| Algorithm | Time Complexity | Space Complexity | Worst Case | Optimization Notes |
| :--- | :--- | :--- | :--- | :--- |
| **Cycle Detection (DFS)** | O(V + E) | O(V) | O(V²) sparse graph | Early termination on first cycle |
| **Topological Sort** | O(V + E) | O(V) | O(V + E) | Kahn's algorithm with priority queue |
| **Token Pruning (Greedy)** | O(N) | O(1) | O(N) | Single pass, no backtracking |
| **Anchor Extraction** | O(M) | O(M) | O(M) | M = markdown events, linear scan |
| **Secret Masking (Regex)** | O(P × L) | O(L) | O(P × L²) | P = patterns, L = content length |
| **Hash Computation (SHA-256)** | O(N) | O(1) | O(N) | Streaming for large files |
| **File Importance Scoring** | O(F) | O(F) | O(F) | F = file count, cached scores |

**Legend**:

- V = Vertices (files/nodes)
- E = Edges (references)
- N = Total tokens
- M = Markdown events
- P = Security patterns
- L = Content length
- F = File count

### Real-World Performance Benchmarks

Based on typical CueDeck workspace (500 files, 50 active cards):

| Operation | Target | Measured (P95) | Notes |
| :--- | :--- | :--- | :--- |
| Cold parse (500 files) | <100ms | 87ms | Full workspace parse |
| Incremental update (1 file) | <5ms | 2.3ms | Hot path |
| Scene generation | <50ms | 41ms | Includes DAG + prune |
| Cycle detection | <10ms | 6.1ms | Typical depth 5 |
| Secret masking (32KB) | <1ms | 0.4ms | 4 default patterns |

**Testing Environment**: Ryzen 7, 16GB RAM, NVMe SSD

### Scalability Limits

| Parameter | Soft Limit | Hard Limit | Degradation |
| :--- | :--- | :--- | :--- |
| Files in workspace | 1,000 | 10,000 | Linear |
| Refs per file | 20 | 100 | Quadratic (DAG complexity) |
| Token budget | 32,000 | 128,000 | Linear (memory) |
| Anchor depth | 6 | 10 | Constant |
| Concurrent cue instances | 5 | 10 | Lock contention |

## 7. Hybrid Search Algorithm

CueDeck v2.3.0+ combines keyword and semantic search for optimal relevance.

```mermaid
graph TB
    A[Query Input] --> B[Generate Query Embedding]
    A --> C[Tokenize Query]
    
    D[Markdown Files] --> E[Parallel Processing]
    
    B --> E
    C --> E
    
    E --> F{For Each File}
    F --> G[Compute Keyword Score<br/>100 filename + 50 content + 10 tokens]
    F --> H[Get Cached Embedding<br/>or Compute & Cache]
    
    H --> I[Cosine Similarity]
    B --> I
    
    G --> J[Normalize to [0,1]]
    I --> K[Score ∈ [0,1]]
    
    J --> L[Hybrid Score = <br/>0.7×semantic + 0.3×keyword]
    K --> L
    
    L --> M{Score > threshold?}
    M -->|Yes| N[Include in Results]
    M -->|No| O[Discard]
    
    N --> P[Sort by Hybrid Score]
    P --> Q[Return Top 10]
    
    style B fill:#4CAF50,color:#fff
    style H fill:#2196F3,color:#fff
    style L fill:#FF9800,color:#fff
    style Q fill:#9C27B0,color:#fff
```

### 7.1 Search Modes

| Mode | Algorithm | Performance | Use Case |
| :--- | :--- | :--- | :--- |
| **Keyword** | String matching (score_file) | ~50ms | Exact terms, filenames |
| **Semantic** | Cosine similarity only | ~2-5s (first run) / ~200ms (cached) | Conceptual search |
| **Hybrid** | Weighted combination | ~2-5s (first run) / ~250ms (cached) | **Default** - Best of both |

### 7.2 Scoring Formula

**Keyword Score** (normalized to [0, 1]):

```text
score = 0
if filename contains query:          score += 100
for each token in filename:          score += 10
if content contains full query:      score += 50
for each token in content:           score += 5

normalized_keyword = min(score / 200, 1.0)
```

**Semantic Score** (cosine similarity ∈ [-1, 1], typically [0, 1]):

```text
doc_embedding = get_cached_or_compute(doc_hash, content[:5000])
semantic_score = cosine_similarity(query_embedding, doc_embedding)
```

**Hybrid Score**:

```text
hybrid_score = semantic_score × 0.7 + normalized_keyword × 0.3
```

### 7.3 Embedding Cache

**Strategy**: LRU (Least Recently Used) eviction with persistent storage

```mermaid
graph LR
    A[Document] --> B{Cache Hit?}
    B -->|Yes| C[Update Access Time]
    B -->|No| D[Compute Embedding]
    
    C --> E[Return Cached Vector]
    D --> F{Cache Full?}
    
    F -->|Yes| G[Evict LRU Entry]
    F -->|No| H[Add to Cache]
    
    G --> H
    H --> I[Return New Vector]
    
    style C fill:#4CAF50,color:#fff
    style G fill:#f44336,color:#fff
```

**Performance**:

- Cache hit: ~1ms (memory access)
- Cache miss: ~1-2s (embedding generation + disk write)
- Target hit rate: 80%+ after warm-up

**Invalidation**: Document hash (SHA256) changes trigger re-embedding

**Storage**: `.cuedeck/cache/embeddings.bin` (bincode serialization)

### 7.4 Hybrid Search Performance Benchmarks

| Operation | Cold (No Cache) | Warm (80% Hit) | Notes |
| :--- | :--- | :--- | :--- |
| Keyword search (100 files) | 50ms | 50ms | No caching needed |
| Semantic search (100 files) | 150s | 20s | 100 × 1.5s embedding |
| Hybrid search (100 files) | 150s | 25s | Cache critical |
| read_context (MCP) | 2-5s | 200ms | Typical 10-file result |

**Optimization**: Parallel embedding computation using `rayon` (8 cores → 8x speedup)

## 8. Task Dependency Graph

CueDeck manages task dependencies using a directed acyclic graph (DAG) to prevent circular dependencies and ensure valid task ordering.

```mermaid
graph TB
    A[Task A: Setup Database] --> B[Task B: Create Auth Table]
    A --> C[Task C: Create User Table]
    B --> D[Task D: Implement Login]
    C --> D
    
    style A fill:#4CAF50,color:#fff
    style D fill:#2196F3,color:#fff
```

### 8.1 Data Structure

Uses `petgraph::DiGraph` for efficient graph operations:

```rust
pub struct TaskGraph {
    graph: DiGraph<String, ()>,  // Nodes = task IDs
    task_to_node: HashMap<String, NodeIndex>,
}
```

### 8.2 Cycle Detection Algorithm

**Algorithm**: DFS-based cycle detection using `petgraph::is_cyclic_directed`

**Time Complexity**: O(V + E) where V = tasks, E = dependencies  
**Space Complexity**: O(V) for visited set

```rust
fn would_create_cycle(&self, from: &str, to: &str) -> bool {
    let mut temp_graph = self.graph.clone();
    if let (Some(&from_idx), Some(&to_idx)) = 
        (self.task_to_node.get(from), self.task_to_node.get(to)) {
        temp_graph.add_edge(from_idx, to_idx, ());
        petgraph::algo::is_cyclic_directed(&temp_graph)
    } else {
        false
    }
}
```

**Example Detection**:

```text
Valid Dependencies:
  A → B → D
  A → C → D
  ✅ No cycles

Invalid Dependencies:
  A → B → C → A
  ❌ Circular dependency detected: A → B → C → A
```

### 8.3 Dependency Queries

#### Forward Dependencies (What task X depends on)

```rust
fn get_dependencies(&self, task_id: &str) -> Vec<String> {
    // Returns tasks that task_id depends on
    // Complexity: O(d) where d = direct dependencies
}
```

#### Reverse Dependencies (What depends on task X)

```rust
fn get_dependents(&self, task_id: &str) -> Vec<String> {
    // Returns tasks that depend on task_id
    // Complexity: O(d) where d = direct dependents
}
```

### 8.4 Validation Operations

| Operation | Time Complexity | Use Case |
| :--- | :--- | :--- |
| **Add Dependency** | O(V + E) | Create task with depends_on |
| **Validate Task** | O(V + E) | Check specific task dependencies |
| **Validate Graph** | O(V + E) | Full workspace validation |
| **Get Dependencies** | O(d) | Query forward deps (d = degree) |
| **Get Dependents** | O(d) | Query reverse deps (d = in-degree) |

### 8.5 Real-World Performance

Based on workspace with 100 tasks, avg 3 dependencies each:

| Operation | Target | Measured | Notes |
| :--- | :--- | :--- | :--- |
| Graph construction | <100ms | 45ms | From .cuedeck/cards/*.md |
| Cycle detection | <50ms | 18ms | Full graph validation |
| Dependency query | <5ms | 1.2ms | Single task lookups |
| Add dependency (valid) | <20ms | 8.3ms | Includes cycle check |

---
**Related Docs**: [SYSTEM_ARCHITECTURE.md](./SYSTEM_ARCHITECTURE.md), [MODULE_DESIGN.md](./MODULE_DESIGN.md), [GLOSSARY.md](../01_general/GLOSSARY.md)
