# Governance Templates

This document provides templates for governance files used by CueDeck's agent system.

## 1. Security Rules Template

**File**: `.cuedeck/security.rules`

```ini
[SECRET_PATTERNS]
# AWS Keys
REGEX: (AKIA|aws_access_key_id)\s*[=:]\s*[A-Za-z0-9/+]{20,}
SEVERITY: CRITICAL
ACTION: block
MESSAGE: AWS credentials detected

[API_KEYS]
REGEX: (api[_-]?key|apikey)\s*[=:]\s*['"][^'"]{16,}['"]
SEVERITY: CRITICAL
ACTION: block

[NAMING_CONVENTIONS]
PATTERN: functions = camelCase
PATTERN: classes = PascalCase
PATTERN: constants = SCREAMING_SNAKE_CASE
PATTERN: private_vars = _camelCase
SEVERITY: WARNING
ACTION: warn

[ARCHITECTURE]
RULE: No circular imports allowed
RULE: All exports must be typed
RULE: Max function length: 100 lines
```

## 2. Role Template: Architect

**File**: `.cuedeck/roles/architect.md`

```markdown
# Architect Role

## Context
You are the system architect. Your decisions shape the entire project structure.

## Responsibilities
- Design system architecture
- Define coding patterns
- Make technology choices
- Review for architectural fit

## Constraints
- Decisions must be documented in ADRs
- Changes require senior review
- No breaking changes without stakeholder approval

## Guidelines
- Consider scalability 3-5 years out
- Document trade-offs
- Align with project roadmap
- Reuse existing patterns

## Available Context
- .cuedeck/governance/architecture.md
- .cuedeck/decisions/ (ADRs)
- Key metrics & performance requirements
```

## 3. Role Template: Reviewer

**File**: `.cuedeck/roles/reviewer.md`

```markdown
# Reviewer Role

## Context
You are the code reviewer. Your role is to ensure quality and consistency.

## Responsibilities
- Review for rule compliance
- Check pattern adherence
- Perform security audit
- Assess performance impact

## Checklist
- [ ] Naming conventions followed
- [ ] No circular dependencies introduced
- [ ] Type safety preserved
- [ ] Tests written for new code
- [ ] No secrets in code
```

## 4. Workflow Template: Feature Development

**File**: `.cuedeck/workflows/feature-spec.md`

```markdown
# Feature Development Workflow

## Stage 1: Specification (Architect)
Input: Feature request
Output: feature.spec.md

**Required context:**
- Project architecture (from architecture.md)
- Similar features (search for patterns)
- Stakeholder requirements
- Performance constraints

**Checklist:**
- [ ] User stories written
- [ ] Acceptance criteria defined
- [ ] Dependencies identified
- [ ] Risk assessment done
- [ ] Design diagram created

**Token budget:** 3000

---

## Stage 2: Planning (Architect + Reviewer)
Input: feature.spec.md
Output: implementation-plan.md

**Required context:**
- Full specification
- Affected modules (max 5)
- Relevant code patterns
- Security rules applicable

**Checklist:**
- [ ] Implementation steps outlined
- [ ] Code structure planned
- [ ] Test strategy defined
- [ ] Security review passed

**Token budget:** 2000

---

## Stage 3: Implementation (Developer)
Input: implementation-plan.md
Output: Code commits

**Required context:**
- Plan
- Code patterns
- Relevant files (working set)
- Test examples

**Checklist:**
- [ ] All acceptance criteria met
- [ ] Tests written
- [ ] Code reviewed by pair
- [ ] No rule violations

**Token budget:** 4000

---

## Stage 4: Review (Reviewer)
Input: All changes
Output: review-feedback.md

**Checks:**
- Rule compliance
- Pattern adherence
- Security audit
- Performance review

**Token budget:** 3000

---

## Stage 5: Integration (Integrator)
Input: Reviewed changes
Output: Merged + deployed

**Checks:**
- No merge conflicts
- All tests pass
- Dependency compatibility
- Rollback plan

**Token budget:** 2000
```

## 5. Workflow Template: Bug Fix

**File**: `.cuedeck/workflows/bug-fix.md`

```markdown
# Bug Fix Workflow

## Stage 1: Triage
- Reproduce the bug
- Identify affected components
- Assess severity

## Stage 2: Analysis
- Review error logs
- Check related code paths
- Identify root cause

## Stage 3: Fix
- Implement minimal fix
- Add regression test
- Update documentation

## Stage 4: Validation
- Run full test suite
- Test in staging
- Get approval
```

## 6. Review Checklist Template

**File**: `.cuedeck/checklists/review.md`

```markdown
# Code Review Checklist

## Security
- [ ] No secrets in changes
- [ ] Input validation present
- [ ] Authorization checks added
- [ ] SQL injection prevention (parameterized queries)

## Code Quality
- [ ] Naming conventions followed
- [ ] Functions under 100 lines
- [ ] No code duplication
- [ ] Tests added
- [ ] Documentation updated

## Architecture
- [ ] No circular dependencies
- [ ] Layer boundaries respected
- [ ] Pattern consistency maintained
- [ ] Type safety preserved

## Performance
- [ ] No N+1 queries
- [ ] No unnecessary rerenders
- [ ] Bundle size impact analyzed
```

## 7. Workflow State Persistence Template

**File**: `.cuedeck/sessions/workflow-state.json`

```json
{
  "workflowId": "feature-dark-mode",
  "status": "in-progress",
  "currentStep": 3,
  "startedAt": "2025-01-15T10:00:00Z",
  "lastUpdate": "2025-01-15T14:30:00Z",
  
  "steps": {
    "specification": {
      "status": "completed",
      "completedAt": "2025-01-15T11:00:00Z",
      "output": "feature-dark-mode.spec.md",
      "approvedBy": "stakeholder"
    },
    "planning": {
      "status": "completed",
      "completedAt": "2025-01-15T12:30:00Z",
      "output": "feature-dark-mode.plan.md"
    },
    "implementation": {
      "status": "in-progress",
      "startedAt": "2025-01-15T13:00:00Z",
      "progress": 0.60,
      "filesModified": ["src/theme.ts", "src/provider.tsx"],
      "lastCheckpoint": "2025-01-15T14:30:00Z"
    },
    "review": {"status": "pending"},
    "integration": {"status": "pending"}
  },
  
  "checkpoints": [
    {
      "step": "implementation",
      "timestamp": "2025-01-15T13:45:00Z",
      "filesCommitted": ["src/theme.ts"],
      "tokensUsed": 1500
    }
  ],
  
  "contextChecksum": "xyz789abc123"
}
```

## 8. Governance Maintenance Schedule

| Period | Task |
|--------|------|
| **Weekly** | Review security.rules against new CVEs, check package updates |
| **Monthly** | Review code for naming violations, analyze workflow metrics |
| **Quarterly** | Major governance review, update architecture guidelines |
| **Yearly** | Comprehensive system audit, technology stack updates |

---
**Related Docs**: [SECURITY.md](../02_architecture/SECURITY.md), [AGENT_PERSONA.md](./AGENT_PERSONA.md), [WORKFLOWS.md](../02_architecture/WORKFLOWS.md)
