# Implementation Patterns

This document captures key implementation patterns and techniques for CueDeck development.

## 1. Token Compression Patterns

### Multi-Stage Compression Pipeline

```text
Input Context
    ↓
Stage 1: Remove Comments (preserve docstrings)
    ↓
Stage 2: Abbreviate Keywords
    async function → async fn
    export const → export
    interface → iface
    ↓
Stage 3: Compress Whitespace
    ↓
Stage 4: Summarize Long Sections (>30 lines)
    → "Implementation details (45 lines)"
    ↓
Stage 5: Reference by Hash (large files)
    → "See src/api.ts#2a1b3c for implementation"
    ↓
Compressed Output (~40% reduction)
```

### Abbreviation Dictionary

| Original | Abbreviation |
|----------|-------------|
| `async function` | `async fn` |
| `export const` | `export` |
| `interface` | `iface` |
| `implementation` | `impl` |
| `public` | `pub` |
| `private` | `priv` |
| `protected` | `prot` |
| `readonly` | `ro` |

## 2. Change Detection Patterns

### Hash-Based Detection

```rust
fn calculate_structure_hash(project: &Project) -> String {
    let mut hash = Sha256::new();
    
    // Sort for deterministic order
    for file in project.files.sorted() {
        let file_hash = hash_file(&file.path);
        hash.update(format!("{}:{}\n", file.path, file_hash));
    }
    
    hash.finalize().to_hex()
}

fn detect_changes(old_hash: &str, new_hash: &str) -> bool {
    old_hash != new_hash
}
```

### Incremental Update Strategy

```text
1. Calculate current structure hash
2. Compare with stored hash
3. If different:
   - Identify affected files (delta)
   - Update only affected file indexes
   - Recompute criticality scores
   - Update structure hash
4. If same:
   - Use cached context (no work needed)
```

## 3. File Importance Scoring

### Scoring Formula

```text
score = (references * 0.3) + (recent_changes * 0.35) + (in_error_stack * 0.35)
```

### Ranking Example

```text
For task "Add auth to API":
1. src/api.ts (referenced 45 times) - MUST INCLUDE
2. src/auth.ts (changed yesterday) - MUST INCLUDE
3. src/types.ts (imported by api.ts) - INCLUDE IF SPACE
4. src/utils.ts (not related) - SKIP
5. docs/api.md (helpful but not critical) - SKIP
```

## 4. Context Staleness Detection

### Staleness Triggers

| Trigger | Action |
|---------|--------|
| Project hash changed | Auto-refresh |
| Working set file modified externally | Alert + refresh |
| 30+ minutes elapsed | Suggest refresh |
| Git branch changed | Full refresh |

### Recovery Flow

```text
Context Stale Detected
    ↓
Load backup: .cuedeck/sessions/[id].backup.json
    ↓
Attempt recovery from last clean state
    ↓
Log incident for analysis
    ↓
Alert: "Context recovered from backup"
    ↓
Require manual /confirm before proceeding
```

## 5. Workflow Step Handoff

### Context Injection Template

```markdown
## From Previous Step: [specification]
- Decision: Use REST API with WebSockets
- Key context: Needs real-time updates
- Current implementation approach: [summary]
- Unresolved issues: [list]

## Continuing with: [implementation]
Your role: Write the actual code
Token budget: 4000
Modified files since start: [list]

You have access to:
- .cuedeck/feature-123.spec.md (previous output)
- Working set: [files changed]
- Relevant skills: [loaded skills]
```

## 6. Rules Parsing Pattern

### security.rules Format

```ini
[SECTION_NAME]
# Comment line
REGEX: pattern_here
SEVERITY: CRITICAL|HIGH|MEDIUM|LOW
ACTION: block|warn|redact
MESSAGE: Human-readable message
FIX_SUGGESTION: How to fix (optional)
LANGUAGE: all|javascript|typescript|python|rust (optional)
```

### Parsing Algorithm

```text
1. Split content by section markers [SECTION_NAME]
2. For each section:
   a. Parse key: value pairs
   b. Compile REGEX patterns
   c. Extract severity and action
   d. Cache by file extension for quick lookup
```

## 7. AST-Based Import Extraction

### TypeScript Pattern

```javascript
const regex = /import\s+(?:{[^}]*}|\*\s+as\s+\w+|\w+.*?)\s+from\s+['"]([^'"]+)['"]/g;
```

### Python Pattern

```javascript
const regex = /^(?:from|import)\s+([^\s]+)/gm;
```

### Rust Pattern

```javascript
const regex = /use\s+(?:crate::)?([^;]+)/g;
```

## 8. Anti-Patterns

### Over-reliance on LLM for Trivial Tasks

- **Problem**: Wastes token budget, slows down workflow, can introduce subtle errors.
- **Solution**: Define clear boundaries for LLM interaction. Use for complex logic, boilerplate generation, or creative problem-solving, not simple refactoring or syntax fixes.

### Lack of Deterministic Output

- **Problem**: LLM output varies, making testing and integration difficult.
- **Solution**: Implement strict output formats (JSON, specific markdown structures), provide examples, and use few-shot prompting. Post-process LLM output to normalize.

### Context Overload

- **Problem**: Providing too much irrelevant context dilutes important information and increases token usage.
- **Solution**: Implement aggressive context compression, intelligent filtering based on task relevance, and multi-stage prompting where context is revealed incrementally.

## 9. Refactoring Recipes

### Extract Function/Component

```text
1. Identify a block of code with a single responsibility.
2. Define clear inputs and outputs for the new function/component.
3. Replace the original block with a call to the new function/component.
4. Update tests to cover the new unit.
```

### Introduce Explaining Variable

```text
1. Identify a complex expression or a magic number.
2. Create a new variable with a descriptive name.
3. Assign the complex expression/magic number to the new variable.
4. Replace the original expression/number with the new variable.
```

### Consolidate Duplicate Code

```text
1. Identify identical or very similar code blocks across multiple locations.
2. Extract the common logic into a shared function, class, or module.
3. Replace the duplicate blocks with calls to the shared logic.
4. Ensure all original use cases are still covered.
```

---
**Related Docs**: [ALGORITHMS.md](./ALGORITHMS.md), [MODULE_DESIGN.md](./MODULE_DESIGN.md), [API_DOCUMENTATION.md](../04_tools_and_data/API_DOCUMENTATION.md)
