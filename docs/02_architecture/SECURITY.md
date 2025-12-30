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

## 5. Extended Secret Patterns

### Database Credentials

```ini
[DATABASE_CREDENTIALS]
# MongoDB connection strings
REGEX: mongodb(\+srv)?:\/\/[^@]*:[^@]*@
SEVERITY: CRITICAL
ACTION: block
MESSAGE: MongoDB credentials detected
FIX_SUGGESTION: Use environment variables

# PostgreSQL passwords
REGEX: postgresql:\/\/[^:]+:[^@]+@
SEVERITY: CRITICAL
ACTION: block

# MySQL passwords  
REGEX: mysql:\/\/[^:]+:[^@]+@
SEVERITY: CRITICAL
ACTION: block
```

### Private Keys

```ini
[PRIVATE_KEYS]
# RSA/ECDSA private keys
REGEX: -----BEGIN (RSA|ECDSA) PRIVATE KEY-----
SEVERITY: CRITICAL
ACTION: block
MESSAGE: Private cryptographic key in source code

# SSH keys
REGEX: -----BEGIN OPENSSH PRIVATE KEY-----
SEVERITY: CRITICAL
ACTION: block
```

### JWT Tokens

```ini
[JWT_TOKENS]
# Real JWT tokens (eyJ format)
REGEX: eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]{40,}
SEVERITY: HIGH
ACTION: redact
MESSAGE: JWT token detected - will be redacted for agent
```

## 6. Code Quality Security Rules

### Unsafe Patterns

```ini
[UNSAFE_PATTERNS]
# Unsafe eval in JavaScript
REGEX: eval\s*\(
SEVERITY: CRITICAL
ACTION: block
LANGUAGE: javascript,typescript
MESSAGE: eval() is unsafe - use safe alternatives

# SQL injection vulnerable patterns
REGEX: query\s*\(\s*["']\s*\+|query\s*\(\s*`[^`]*\$\{
SEVERITY: CRITICAL
ACTION: block
MESSAGE: SQL injection vulnerability - use parameterized queries
```

### Dangerous Configuration

```ini
[DANGEROUS_CONFIG]
PATTERN_FILE: .env
DANGEROUS: DEBUG=true
SEVERITY: CRITICAL
ACTION: block
MESSAGE: Debug mode enabled in production environment

PATTERN_FILE: .env.production
DANGEROUS: SSL_VERIFY=false
SEVERITY: CRITICAL
ACTION: block
```

---
**Related Docs**: [SYSTEM_ARCHITECTURE.md](../02_architecture/SYSTEM_ARCHITECTURE.md), [TROUBLESHOOTING.md](../05_quality_and_ops/TROUBLESHOOTING.md), [GOVERNANCE_TEMPLATES.md](../03_agent_design/GOVERNANCE_TEMPLATES.md), [ARCHITECTURE_RULES.md](./ARCHITECTURE_RULES.md)
