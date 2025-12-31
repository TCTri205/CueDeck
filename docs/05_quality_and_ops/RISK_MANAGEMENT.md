# Risk Management

This document outlines potential risks and mitigations for the CueDeck system.

## 1. Critical Risks

| Risk | Impact | Likelihood | Mitigation |
| :--- | :--- | :--- | :--- |
| **Context Corruption** | CRITICAL | Medium | Weekly integrity checksums, git-based rollback |
| **Token Leakage (Secrets)** | CRITICAL | High | Aggressive regex in security rules, human review before external send |
| **Stale Context Bugs** | HIGH | Medium | Auto-refresh every 5min, `/refresh` command |
| **Update Failure** | HIGH | Low | Dual-partition implementation, atomic swap, checksum verification |

## 2. Operational Risks

| Risk | Impact | Likelihood | Mitigation |
| :--- | :--- | :--- | :--- |
| **Performance Degradation** | MEDIUM | Medium | Token budgets, lazy loading, LRU caching |
| **Workflow State Loss** | MEDIUM | Low | Persistent storage in `.cuedeck/sessions/`, session recovery |
| **Cache Invalidation** | MEDIUM | Medium | SHA256-based change detection, lazy GC |

## 3. Development Risks

| Risk | Impact | Likelihood | Mitigation |
| :--- | :--- | :--- | :--- |
| **Breaking Changes** | HIGH | Medium | Versioned config schema, migration scripts |
| **Dependency Conflicts** | MEDIUM | Low | Lock file management, compatibility testing |
| **Feature Creep** | MEDIUM | High | Strict roadmap adherence, MVP focus |

## 4. Risk Priority Matrix

Visualizing risk based on Impact vs Likelihood.

| Likelihood ↓ / Impact → | Low | Medium | Critical |
| :--- | :--- | :--- | :--- |
| **High** | Feature Creep (Monitor) | **Token Leakage** (Mitigate) | **Data Loss** (Prevent) |
| **Medium** | Minor Bugs (Accept) | Performance (Optimize) | **Context Corruption** (Recover) |
| **Low** | UI Glitches (Fix) | Network Fail (Retry) | **Update Failure** (Rollback) |

## 5. Disaster Recovery (DR)

### Scenario: Repository Corruption

If `.cuedeck/` becomes unusable:

1. **Nuke**: `rm -rf .cuedeck/`
2. **Re-init**: `cue init`
3. **Restore**: `git checkout HEAD -- .cuedeck/` (if committed)
4. **Re-index**: `cue scene --force`

### Scenario: Token Budget Exhaustion

If the project grows too large for context (Cost Spike):

1. **Analyze**: `cue doctor --stats` to find large files.
2. **Configure**: Add huge files to `.cuedeckignore` or `[parser.exclude]`.
3. **Optimze**: Switch to `tiktoken-rs` exact counting.
4. **Partition**: Split project into smaller workspaces.

## 6. Risk Monitoring

### Health Checks

```text
Automated checks (every 5 minutes):
- [ ] Cache consistency verified
- [ ] Session files valid
- [ ] Secret patterns updated
- [ ] Performance within bounds
```

### Incident Response

1. **Detection**: Automated alerts from integrity checker
2. **Assessment**: Check severity level
3. **Containment**: Rollback to last known good state
4. **Recovery**: Apply fix and validate
5. **Post-mortem**: Document and update mitigations

## 7. Rollback Strategy

### File-Level Rollback

```text
.cuedeck/
├── .backup/                    # Automatic backups
│   ├── metadata.json.bak      # Previous cache state
│   ├── sessions/              # Session snapshots
│   └── cards/                 # Card history
```

### Command: `cue doctor --repair`

- Validates all cached hashes
- Removes orphaned entries
- Rebuilds corrupted metadata
- Reports recovery status

---
**Related Docs**: [SECURITY.md](../02_architecture/SECURITY.md), [TESTING_STRATEGY.md](./TESTING_STRATEGY.md), [TROUBLESHOOTING.md](./TROUBLESHOOTING.md)
