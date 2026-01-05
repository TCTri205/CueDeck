# Security Rules for Agentic Development

> [!CAUTION] **Zero Tolerance**
>
> Violations of these rules are **automatic PR rejection**. No exceptions.

## Rule 1: Secret Management

### Forbidden Patterns

Agent MUST reject any code containing:

```rust
// ❌ FORBIDDEN
let api_key = "sk-1234567890abcdef";
const TOKEN: &str = "ghp_xxxxxxxxxxxxx";
std::env::set_var("API_KEY", "secret");
```

### Approved Patterns

```rust
// ✅ APPROVED
use std::env;

let api_key = env::var("API_KEY")
    .wrap_err("API_KEY environment variable not set")?;
```

### Detection Regex

Agent MUST scan for:

- `(token|key|secret|password)\s*=\s*["'][^"']+["']` (case-insensitive)
- Hard-coded credentials in config files

### Action on Detection

```text
🚨 SECURITY VIOLATION DETECTED

Pattern: Hard-coded API key
Location: src/auth/client.rs:42
Rule: SECURITY_RULES.md Rule 1

BLOCKING: Cannot proceed until fixed.
Suggested fix: Use environment variable or .env file
```

## Rule 2: Logging Sensitive Data

### Forbidden

```rust
// ❌ FORBIDDEN
tracing::info!("User logged in: {:?}", user);  // Contains email, password hash
log::debug!("Request: {:?}", http_request);     // Contains Authorization header
```

### Approved

```rust
// ✅ APPROVED
tracing::info!("User logged in", user_id = %user.id);
log::debug!("Request to {}", http_request.path());  // No headers
```

### Allowlist for Logging

**Safe fields**:

- User ID (numeric/UUID)
- Timestamps
- HTTP status codes
- File paths (sanitized)

**Forbidden fields**:

- Passwords (plain or hashed)
- API keys/tokens
- Email addresses (use hash)
- Full request/response bodies

## Rule 3: Input Validation

### Required for All External Inputs

```rust
pub fn create_task(title: &str) -> Result<Task> {
    // 1. Length validation
    if title.is_empty() || title.len() > 200 {
        return Err(ValidationError::InvalidLength);
    }
    
    // 2. Character whitelist (if applicable)
    if !title.chars().all(|c| c.is_alphanumeric() || " _-".contains(c)) {
        return Err(ValidationError::InvalidCharacters);
    }
    
    // 3. Path traversal check (for file operations)
    if title.contains("..") {
        return Err(ValidationError::PathTraversal);
    }
    
    // Safe to use
    Ok(Task::new(title))
}
```

### Database Inputs

**ALWAYS use parameterized queries**:

```rust
// ❌ FORBIDDEN (SQL injection risk)
let query = format!("SELECT * FROM tasks WHERE title = '{}'", user_input);

// ✅ APPROVED
conn.query_row(
    "SELECT * FROM tasks WHERE title = ?1",
    params![user_input],
    |row| { ... }
)?;
```

## Rule 4: Rate Limiting

### All Network Operations

```toml
[rate_limit]
max_requests_per_minute = 60
burst_size = 10
```

```rust
use governor::{Quota, RateLimiter};

let limiter = RateLimiter::direct(
    Quota::per_minute(nonzero!(60u32))
);

limiter.until_ready().await;
make_request().await?;
```

## Rule 5: Dependency Security

### Update Policy

- **Critical vulnerabilities**: Fix within 24h
- **High**: Fix within 1 week  
- **Medium/Low**: Fix in next release

### Check on Every PR

```bash
cargo audit
cargo outdated
```

## Rule 6: File System Operations

### Path Sanitization

```rust
use std::path::{Path, PathBuf};

pub fn safe_path(base: &Path, user_input: &str) -> Result<PathBuf> {
    let path = base.join(user_input);
    
    // Ensure path is within base directory
    let canonical = path.canonicalize()
        .wrap_err("Invalid path")?;
    
    if !canonical.starts_with(base) {
        return Err(SecurityError::PathTraversal);
    }
    
    Ok(canonical)
}
```

### Forbidden Operations

```rust
// ❌ FORBIDDEN - No user input validation
std::fs::remove_file(&user_provided_path)?;

// ✅ APPROVED - Validated and sandboxed
let safe = safe_path(&workspace_root, user_input)?;
std::fs::remove_file(&safe)?;
tracing::warn!("Deleted file", path = %safe.display());
```

## Rule 7: External Command Execution

### Blocklist

Agent MUST reject commands containing:

- `rm -rf` / `del /f /s` (destructive file operations)
- `curl | sh` / `wget | bash` (piped execution)
- `sudo` / `su` (privilege escalation)
- `eval` (code injection risk)
- Direct database commands (`psql`, `mysql`, `mongo`)

### Safe Execution Pattern

```rust
use std::process::Command;

pub fn run_safe_command(program: &str, args: &[&str]) -> Result<String> {
    // Whitelist allowed programs
    const ALLOWED: &[&str] = &["git", "cargo", "cue"];
    
    if !ALLOWED.contains(&program) {
        return Err(SecurityError::UnauthorizedCommand);
    }
    
    let output = Command::new(program)
        .args(args)
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

## Verification Checklist

Before merging, agent MUST confirm:

- [ ] No hard-coded secrets (run regex scan)
- [ ] No sensitive data in logs (manual review of `tracing::` calls)
- [ ] All external inputs validated
- [ ] Parameterized queries for database
- [ ] Rate limiting on network calls
- [ ] File paths sanitized and sandboxed
- [ ] External commands whitelisted
- [ ] `cargo audit` passes

## Incident Response

If security violation is detected:

1. **STOP** - Halt all code generation
2. **ALERT** - Notify human with specific violation details
3. **SUGGEST** - Provide secure alternative implementation
4. **VERIFY** - After fix, re-scan entire changeset

---

**Related Docs**:

- [APPROVED_LIBRARIES.md](../03_agent_design/APPROVED_LIBRARIES.md)
- [CONTRIBUTING.md](../01_general/CONTRIBUTING.md)
- [ENGINEERING_STANDARDS.md](../03_agent_design/ENGINEERING_STANDARDS.md)
