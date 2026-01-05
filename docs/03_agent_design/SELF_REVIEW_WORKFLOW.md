# Agent Self-Review Workflow

> [!IMPORTANT] **Mandatory Protocol**
>
> All agents must execute this workflow **before** requesting human review.
> This transforms the static [Safety Checklist](../../.cuedeck/prompts/safety_checklist.md) into actionable steps.

## Overview

**Goal**: Minimize "Review Fatigue" by ensuring code is 100% compliant, tested, and documented before human eyes see it.

**When to Run**:

- After completing implementation
- Before submitting a Pull Request
- Before `notify_user` for final approval

---

## Phase 1: Pre-Flight Checks

**Objective**: Ensure basic architectural alignment and workspace limits.

### Step 1.1: Verify Scope

```bash
# Check files changed
git status --short
```

- [ ] **Files**: Are these the *only* files I intended to change?
- [ ] **Unexpected**: Any `*.lock`, `.cuedeck/cache`, or unrelated files? -> **Revert them.**

### Step 1.2: Architecture Alignment

```bash
# Verify no cross-layer violations (e.g., cli importing from core)
# (Manual check of imports in changed files)
```

- [ ] **Rule Check**: Did I read [ARCHITECTURE.md](../02_architecture/ARCHITECTURE.md)?
- [ ] **Domain**: Did I modify `usage` or `billing`? -> **Check [CORE_DOMAIN.md](../02_architecture/CORE_DOMAIN.md)**

---

## Phase 2: Code Quality Verification

**Objective**: Zero warnings, zero errors, full test coverage.

### Step 2.1: Syntax & Compilation

```bash
cargo check --workspace --all-features
```

- [ ] **Must pass** without errors.

### Step 2.2: Lints & Complexity

```bash
# -D warnings promotes warnings to errors
cargo clippy --workspace --all-features -- -D warnings
```

- [ ] **Complexity**: If this fails on cognitive complexity, **refactor** (extract functions).
- [ ] **Style**: Fix all suggestions.

### Step 2.3: Formatting

```bash
cargo fmt --all -- --check
```

- [ ] If fails: Run `cargo fmt --all` to fix.

### Step 2.4: Testing

```bash
# Fast unit tests
cargo test --workspace --lib

# Doc tests
cargo test --workspace --doc
```

- [ ] **All Pass**: No flaky tests allowed.
- [ ] **Coverage**: Did I add tests for new logic?

### Step 2.5: Security Scan

```bash
# Check for hardcoded secrets
rg -i '(token|key|secret|password)\s*=\s*["\'][^"\']+["\']' --glob '!*.lock'
```

- [ ] **No Hits**: If found, move to env vars.

---

## Phase 3: Documentation & Communication

**Objective**: Ensure the "Why" is clear.

### Step 3.1: Public API Docs

- [ ] Check all `pub` functions in changed files.
- [ ] Do they have `///` comments explaining usage?

### Step 3.2: Artifact Update

- [ ] Did I update `task.md`?
- [ ] Did I update `walkthrough.md`?

---

## Phase 4: Bug Detection Patterns

**Aggressively hunt for these common AI-generated bugs:**

### 🔴 The "Silent Failure"

**Pattern**: `Result` returned but ignored.

```rust
// ❌ Bad
let _ = file.write(b"content");

// ✅ Good
file.write(b"content")?;
```

### 🔴 The "Infinite Loop"

**Pattern**: `loop` or `while` without clear exit or timeout.
**Fix**: Add timeouts or loop counters (see [CIRCUIT_BREAKER.md](../02_architecture/CIRCUIT_BREAKER.md)).

### 🔴 The "Partial Write"

**Pattern**: Writing to file without `flush()` or atomic move.
**Fix**: Use `tempfile` crate pattern for atomic saves.

### 🔴 The "Time Bomb"

**Pattern**: Hardcoded dates or assumptions about timezone.
**Fix**: Use `chrono::Utc::now()` and relative durations.

---

## Phase 5: Handoff Decision

**Can I proceed to Human Review?**

1. **YES** if:
   - All Phases 1-3 pass checks.
   - Code builds, tests pass, lints pass.
   - Documentation is updated.

2. **NO (Escalate)** if:
   - Architecture violation required to fix bug.
   - Ambiguous requirement.
   - Security risk identified.

**Escalation Template**:

```text
STOP: Unable to complete Self-Review using SELF_REVIEW_WORKFLOW.md.
Reason: [Architecture Conflict / Ambiguity / Security]
Details: ...
Requesting guidance.
```

---

## Related Docs

- [Safety Checklist](../../.cuedeck/prompts/safety_checklist.md)
- [COMPLEXITY_METRICS.md](../05_quality_and_ops/COMPLEXITY_METRICS.md)
- [SECURITY_RULES.md](../04_security/SECURITY_RULES.md)
