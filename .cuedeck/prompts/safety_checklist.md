# Agent Safety Checklist

**Purpose**: Pre-merge verification for AI-generated code changes

---

## 1. Scope Validation

- [ ] Operation affects only intended files (list files explicitly)
- [ ] No accidental changes to unrelated modules
- [ ] Total lines changed within agreed limit (default: 500)
- [ ] No modifications to forbidden patterns:
  - [ ] `.cuedeck/.cache/*` (system-managed)
  - [ ] `*.lock` files (package manager)
  - [ ] Core domain without enhanced docs

---

## 2. Architecture Compliance

- [ ] Read `ARCHITECTURE.md` and `ARCHITECTURE_RULES.md` before coding
- [ ] Module boundaries respected (no cross-layer imports)
- [ ] Error handling uses `miette::Result<T>`
- [ ] Naming conventions followed (see `ENGINEERING_STANDARDS.md`)
- [ ] If core domain modified → Enhanced documentation provided

---

## 3. Reversibility

- [ ] Changes are Git-tracked (committed to branch)
- [ ] Rollback plan exists and tested:

  ```bash
  git revert <commit>
  cargo test
  ```

- [ ] No irreversible data deletion without backup
- [ ] Database migrations have down() function

---

## 4. Security

- [ ] No hard-coded secrets (scan for `sk-`, `api_key =`, etc.)
- [ ] No sensitive data in logs (passwords, tokens, emails)
- [ ] Input validation for all external inputs
- [ ] SQL queries use parameterized statements (no string formatting)
- [ ] File paths sanitized (no `..` traversal)
- [ ] External commands whitelist-checked

**Security Scan**:

```bash
# Check for hardcoded secrets
rg -i '(token|key|secret|password)\s*=\s*["\'][^"\']+["\']' --glob '!*.lock'

# Check for sensitive logging
rg 'log|tracing|debug' | rg '(password|token|api_key)'
```

---

## 5. Testing

- [ ] Unit tests cover changed logic
- [ ] Integration tests pass
- [ ] No manual testing needed **OR** manual steps documented
- [ ] Property-based tests for complex logic (if applicable)
- [ ] Regression test added for bug fixes

**Test Commands**:

```bash
# Run all tests
cargo test --workspace

# Run specific package
cargo test --package cue_core

# Run with coverage
cargo tarpaulin --out Html
```

---

## 6. Performance

- [ ] No obvious performance regressions
- [ ] Benchmarks run (if performance-critical)
- [ ] No new N+1 queries
- [ ] Cache invalidation strategy defined

---

## 7. Documentation

- [ ] Code comments explain **WHY**, not WHAT
- [ ] Public APIs have doc comments
- [ ] If breaking change → Migration guide provided
- [ ] Updated `CHANGELOG.md` (for user-facing changes)

---

## 8. Communication

- [ ] Explained **WHY** this approach (alternatives considered)
- [ ] Listed trade-offs clearly
- [ ] Highlighted potential risks
- [ ] Cited sources (file paths, doc anchors)

---

## 9. Human Confirmation (High-Risk Only)

**Trigger if**:

- Deleting files
- Schema migration
- Breaking API changes
- Core domain modifications

**Required**:

- [ ] Human explicitly approved operation
- [ ] Rollback plan communicated
- [ ] Risk level estimated (Low/Medium/High/Critical)

---

## 10. Final Verification

- [ ] `cargo clippy` passes (no warnings)
- [ ] `cargo fmt` applied
- [ ] No `TODO` or `FIXME` without issue links
- [ ] PR description includes:
  - Problem statement
  - Solution approach
  - Test strategy
  - Rollback plan (if needed)

---

## Example: Completed Checklist

```markdown
## Change: Add JWT validation to auth module

### 1. Scope Validation
- [x] 3 files modified: login.rs, middleware.rs, login_test.rs
- [x] Total: +95 -15 lines (within 500 limit)
- [x] No core domain affected

### 2. Architecture Compliance
- [x] Read ARCHITECTURE_RULES.md
- [x] Business logic in cue_core::auth (compliant)
- [x] Uses miette::Result<AuthToken>

### 3. Reversibility
- [x] Committed to branch `feat/jwt-validation`
- [x] Rollback: `git revert abc123`

### 4. Security
- [x] No secrets (uses env var JWT_SECRET)
- [x] Input validation: token length, format, signature
- [x] No SQL (uses rusqlite prepared statements)

### 5. Testing
- [x] Unit: test_jwt_valid, test_jwt_expired, test_jwt_malformed
- [x] Integration: test_login_with_jwt
- [x] All tests pass

### 6. Performance
- [x] JWT validation adds ~2ms (acceptable)
- [x] No N+1 queries

### 7. Documentation
- [x] Doc comments on validate_jwt()
- [x] Updated docs/AUTHENTICATION.md

### 8. Communication
- [x] Approach: RS256 (public key crypto)
- [x] Alternative: HS256 - rejected (symmetric key risk)
- [x] Risk: Clock skew (mitigated by 30s leeway)

### 9. Human Confirmation
- [n/a] Not high-risk (no deletion, no schema change)

### 10. Final Verification
- [x] cargo clippy --all-targets -- -D warnings
- [x] cargo fmt --check
- [x] No TODOs
- [x] PR #143 created with full description
```

---

**Related Docs**:

- [SECURITY_RULES.md](../04_security/SECURITY_RULES.md)
- [ENGINEERING_STANDARDS.md](./ENGINEERING_STANDARDS.md)
- [CORE_DOMAIN.md](../02_architecture/CORE_DOMAIN.md)
