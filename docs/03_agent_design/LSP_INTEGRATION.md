# LSP Integration Guide for Agents

**Purpose**: Empower AI Agents to "see" code structure, types, and errors using Language Server Protocol (LSP), rather than just reading raw text.

---

## 1. Why LSP?

Text-based understanding (RegEx/Reading) vs. Semantic understanding (LSP):

| Feature | Text-Based (Standard) | LSP-Enhanced (Agentic) |
| :--- | :--- | :--- |
| **Context** | "I see the string 'Foo'" | "I see class 'Foo' inheriting from 'Bar'" |
| **Validation** | Guess if it compiles | Know immediately if it breaks (Diagnostics) |
| **Refactoring** | Grep and pray | Rename Symbol (guaranteed safe) |
| **Discovery** | Scroll to find function | "Go to Definition" |

**Goal**: Agents should act like an IDE, not a text editor.

---

## 2. Agent-LSP Interface (MCP Tools)

CueDeck exposes LSP capabilities to agents via Model Context Protocol (MCP) tools.

### A. `lsp_hover`

Get type information and documentation for the symbol at a specific location.

- **Use Case**: "What does this variable `config` actually contain?"
- **Input**: `{ path: "src/main.rs", line: 42, character: 15 }`
- **Output**:

  ```markdown
  **Type**: `struct Config`
  **Docs**: "Global application configuration loaded from toml."
  ```

### B. `lsp_definition`

Navigate to where a symbol is defined.

- **Use Case**: "Where is `User` struct defined?"
- **Input**: `{ path: "src/main.rs", line: 10, character: 20 }`
- **Output**: `{ path: "src/models/user.rs", line: 5 }`

### C. `lsp_references`

Find all usages of a symbol (Impact Analysis).

- **Use Case**: "If I change this function signature, who breaks?"
- **Input**: `{ path: "src/api.rs", line: 50, character: 10 }`
- **Output**: List of 15 locations calling this function.

### D. `lsp_diagnostics`

Get current compilation errors and warnings.

- **Use Case**: "Did my last edit fix the bug?"
- **Input**: `{ path: "src/main.rs" }`
- **Output**:

  ```json
  [
    {
      "severity": "Error",
      "line": 42,
      "message": "mismatched types: expected `String`, found `u32`"
    }
  ]
  ```

---

## 3. Recommended Workflows

### Scenario 1: Debugging a Compilation Error

**Traditional Agent**:

1. Reads file.
2. Guesses the fix.
3. Runs `cargo build`.
4. Sees error.
5. Repeats (slow loop).

**LSP Agent**:

1. Calls `lsp_diagnostics` -> Sees "Expected String".
2. Calls `lsp_hover` on variable -> Sees it's `u32`.
3. Applies fix (`.to_string()`).
4. Calls `lsp_diagnostics` again -> Empty (Success!).
5. **Impact**: Zero CLI build commands needed until final verification.

### Scenario 2: Safe Refactoring

**Traditional Agent**:

1. Search/Replace "User" -> "Customer".
2. Accidentally renames "User" in comments or strings.

**LSP Agent**:

1. Calls `lsp_references` on `User` struct.
2. Verifies all 12 usages are code refs.
3. Applies change to specific lines.
4. **Impact**: No accidental string replacements.

---

## 4. Integration Guide

### Requirements

To enable LSP for agents:

1. **LSP Server**: Must be installed (e.g., `rust-analyzer`, `tsserver`, `gopls`).
2. **CueDeck MCP**: Enable the LSP capability in `config.toml`.

```toml
[agent.capabilities]
lsp_enabled = true
lsp_timeout_ms = 5000

[agent.lsp.servers]
rust = "rust-analyzer"
typescript = "typescript-language-server --stdio"
```

### Agent Prompt Instructions

Add to `PROMPTS_AND_INSTRUCTIONS.md` (System Prompt):

> **LSP TOOL USAGE**:
> Before reading a whole file to find a definition, use `lsp_definition`.
> Before running a build to check syntax, use `lsp_diagnostics`.
> Trust LSP types over your own inference.

---

## 5. Future Roadmap

- **Code Action Support**: Allow agents to trigger "Quick Fixes" (e.g., "Import missing module").
- **Symbol Search**: `lsp_workspace_symbol` for fuzzy finding types.
- **Inlay Hints**: Inject type hints into `view_file` output automatically.

---

## Related Docs

- [TOOLS_SPEC.md](../04_tools_and_data/TOOLS_SPEC.md) - MCP Schema definitions.
- [ENGINEERING_STANDARDS.md](../03_agent_design/ENGINEERING_STANDARDS.md) - Code quality rules.
- [PROMPTS_AND_INSTRUCTIONS.md](../03_agent_design/PROMPTS_AND_INSTRUCTIONS.md) - Agent guidelines.
