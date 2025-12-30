# Security Specification

## 1. Threat Model

CueDeck operates as a **local-first** tool, but it interfaces with external LLMs. The primary threat is **accidental secret leakage**.

### Attack Surface

| Vector | Risk | Mitigation |
| :--- | :--- | :--- |
| **LLM Context** | API keys sent to cloud LLM. | Secret Masking (Regex Guard). |
| **Clipboard** | Sensitive data copied. | User Awareness (Cannot prevent). |
| **MCP Stdout** | Secrets in JSON response. | Secret Masking applied to ALL outputs. |
| **Log Files** | Secrets in debug logs. | Logs on `stderr` are filtered (Level: INFO+). |

## 2. The Secret Guard

The final filter before any content leaves the system.

### Implementation

```rust
// Pseudocode
fn mask_secrets(input: &str, patterns: &[Regex]) -> String {
    let mut output = input.to_string();
    for pattern in patterns {
        output = pattern.replace_all(&output, "$prefix***").to_string();
    }
    output
}
```

### Default Patterns (`config.toml`)

```toml
[security]
secret_patterns = [
    "(sk-[a-zA-Z0-9]{20,})",   # OpenAI
    "(ghp_[a-zA-Z0-9]{36})",   # GitHub PAT
    "(AKIA[0-9A-Z]{16})",      # AWS Access Key
    "(xox[baprs]-[a-zA-Z0-9-]+)" # Slack Tokens
]
```

### Verification (Unit Tests)

**File**: `crates/cue_core/src/security.rs`

```rust
#[test]
fn test_masks_openai_key() {
    let input = "My key is sk-1234567890abcdef1234567890.";
    let output = mask_secrets(input);
    assert_eq!(output, "My key is sk-***.");
}

#[test]
fn test_ignores_short_keys() {
    let input = "Short sk-123 is not a real key.";
    let output = mask_secrets(input);
    assert_eq!(output, input); // Unchanged
}
```

## 3. Sandbox Rules

CueDeck **does not** execute arbitrary code. Its security model is based on:

- **Read-Only Graph**: The DAG resolver only reads files; it cannot write.
- **Path Restriction**: MCP tools are restricted to the Workspace root (`cwd`).
- **No Network**: The `cue_core` library has no network dependencies; it's pure file I/O.

## 4. Data Isolation

- **Cache**: `.cuedeck/.cache/` is Git-ignored and contains no secrets (only hashes and counts).
- **Logs**: `.cuedeck/logs/mcp.log` should be added to `.gitignore` by `cue init`.

---
**Related Docs**: [SYSTEM_ARCHITECTURE.md](../02_architecture/SYSTEM_ARCHITECTURE.md), [TROUBLESHOOTING.md](../05_quality_and_ops/TROUBLESHOOTING.md)
