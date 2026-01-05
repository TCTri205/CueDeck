# Master Verification Plan

> [!IMPORTANT] **Strategy Definition**
>
> This document defines **HOW** we verify software quality in specific, executable terms.
> It unifies Unit Testing, Integration Testing, and Governance Compliance into a single strategy.

## 1. Verification Layers

We employ a "Defense in Depth" strategy with 4 layers of verification.

| Layer | Scope | Frequency | Mechanism | Owner |
| :--- | :--- | :--- | :--- | :--- |
| **L1: Static Governance** | Code Style, Security, Complexity | Pre-Commit / Save | `cargo clippy`, Regex | Agent/Dev |
| **L2: Unit Tests** | Individual functions/structs | Continuous | `cargo test --lib` | Agent/Dev |
| **L3: Integration** | Module interactions, Flows | Pre-Merge | `cargo test --test '*'` | CI Pipeline |
| **L4: Manual Handoff** | High-risk changes, UX | Pre-Release | Human Review | Human |

---

## 2. L1: Static Governance (The "Gatekeeper")

Before code is ever compiled or tested, it must pass governance checks.

### 2.1 Security Scanning

**Source**: [`SECURITY_RULES.md`](../04_security/SECURITY_RULES.md)

- **Check**: No hardcoded secrets (Regex scan)
- **Check**: No sensitive logging (Review `tracing` calls)
- **Tool**: `rg` (ripgrep) + `cargo audit`

### 2.2 Complexity Enforcement

**Source**: [`COMPLEXITY_METRICS.md`](./COMPLEXITY_METRICS.md)

- **Cyclomatic/Cognitive**: Max 25 (via Clippy)
- **Function Length**: Max 100 lines (via Clippy)
- **Tool**: `cargo clippy -- -D warnings`

### 2.3 Architecture Compliance

**Source**: [`ARCHITECTURE.md`](../02_architecture/ARCHITECTURE.md)

- **Check**: No forbidden dependencies (e.g., Core depends on CLI)
- **Check**: Domain rules respected
- **Tool**: Manual Self-Review + `cargo check`

---

## 3. L2: Unit Testing (The "Bedrock")

**Standard**: [`ENGINEERING_STANDARDS.md`](../03_agent_design/ENGINEERING_STANDARDS.md)

### 3.1 Requirements

- **Coverage**: Logic branches, error conditions, edge cases.
- **Isolation**: No network, no database (use mocks/in-memory).
- **Speed**: Must run in < 100ms.

### 3.2 Location

- **Private logic**: In `src/lib.rs` (submodule) or same file.
- **Public API**: In `tests/` directory (integration style) or `src/lib.rs`.

**Example**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_risk_returns_correct_score() {
        let input = RiskInput::new(5, 10);
        assert_eq!(calculate_risk(input), RiskLevel::High);
    }
}
```

---

## 4. L3: Integration Testing (The "Workflow")

Tests real-world scenarios using the actual components (or persistent mocks).

### 4.1 Scope

- **Database**: Use in-memory SQLite (`:memory:`) or tempfile DB.
- **API**: Test full request/response cycle.
- **CLI**: Test command parsing and output.

### 4.2 Test Helpers

Use `cue_test_helpers` to reduce boilerplate.

```rust
use cue_test_helpers::{setup_test_env, TestContext};

#[tokio::test]
async fn test_full_task_creation_flow() {
    let ctx = setup_test_env().await;
    let task = ctx.create_task("New Feature").await.unwrap();
    assert_eq!(task.status, TaskStatus::Todo);
}
```

---

## 5. L4: Manual Handoff Verification

Required for **High Risk** changes (defined in [`RISK_MANAGEMENT.md`](./RISK_MANAGEMENT.md)).

### 5.1 Procedure

1. **Agent Request**: Agent generates `SELF_REVIEW_WORKFLOW.md` report.
2. **Human Review**: Human checks architecture alignment and business logic.
3. **Approval**: Human explicitly types "Approved".

---

## 6. Automated Workflow (The "Pipeline")

We will automate checking layers L1-L3 via a single accessible script.

### Script: `check_platforms.ps1` (Planned)

This script will run the full gauntlet:

1. **Governance**: Check for secrets, forbidden patterns.
2. **Quality**: Run `cargo clippy` (deny warnings).
3. **Tests**: Run `cargo test`.
4. **Docs**: Verify key files exist.

**Usage**:

```powershell
./scripts/check_governance.ps1
```

---

## 7. Metrics & Reporting

We track the following to ensure quality trends upward:

| Metric | Target | tracked By |
| :--- | :--- | :--- |
| **Test Pass Rate** | 100% | CI |
| **Clippy Warnings** | 0 | CI |
| **Audit Vulnerabilities** | 0 (Critical/High) | CI |
| **Documentation Coverage** | 100% (Public API) | Lints |

---

## Version History

- **v1.0**: Initial strategy definition.
