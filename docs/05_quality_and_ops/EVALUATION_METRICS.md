# Evaluation Metrics

## 1. Performance Baselines (Bench via `criterion`)

| Metric | Target | Failure Threshold |
| :--- | :--- | :--- |
| **Hot Path** (Inc. Update) | **< 5ms** | > 20ms |
| **Cold Start** (100 files) | **< 1s** | > 3s |
| **Memory Footprint** | **< 50MB** | > 200MB |

## 2. Reliability KPIs

- **Cache Hit Rate**: Should be > 95% during normal editing sessions.
- **Cache Rot**: 0%. (Measured by: Does `hash(file)` match `metadata.json` on next read? If no, logic bug).
- **Cycle Rejection**: 100%. Any circular dependency must result in an immediate Error, never a hang.

## 3. Agent Quality (Harder to Measure)

- **Context Density**: (Relevant Lines / Total Lines in Scene).
  - *Optimization*: Tweaking the "Pruning Priority" in `SYSTEM_ARCHITECTURE.md`.
- **Hallucination Rate**: (Invalid File References / Total References).
  - *Fix*: Strict checking in `read_context`.

## 4. Token Efficiency (Extended)

### Baseline & Targets

| Metric | Baseline | Target | Measurement |
| :--- | :--- | :--- | :--- |
| **Avg tokens per session** | Week 1 measurement | **-40%** by Week 6 | Per workflow type |
| **Context loading time** | - | **< 1s** | Benchmark |
| **Memory usage** | - | **< 500MB** | Runtime profiling |
| **Rule matching speed** | - | **< 100ms per file** | Benchmark |

### Token Tracking

```text
Per-task budgets:
├── feature-development: 6000 tokens
│   ├── project context: 2000
│   ├── relevant files: 2500
│   ├── rules & roles: 1000
│   └── free: 500
│
├── bug-fix: 4000 tokens
│   ├── error message: 500
│   ├── related code: 2000
│   ├── similar bugs: 1000
│   └── free: 500
│
└── refactoring: 5000 tokens
    ├── full module: 2500
    ├── dependencies: 1500
    ├── standards: 800
    └── free: 200
```

## 5. Context Accuracy

- **Target**: ≥ 95% context correctness.
- **Measurement**:
  - Manual review of agent decisions.
  - Catch incorrect assumptions.
  - Measure forgetting events.

## 6. Productivity Tracking

- **Track**: Time to complete task from start to final commit.
- **Baseline**: Manual workflow.
- **Target**: Agent workflow **60% faster**.

## 7. Measurement Cadence

```text
Week 2, 4, 6, 8, 10:
- Review metrics
- Identify bottlenecks
- Adjust token budgets
- Optimize slow paths
- Update documentation
```

---
**Related Docs**: [TESTING_STRATEGY.md](./TESTING_STRATEGY.md), [SYSTEM_ARCHITECTURE.md](../02_architecture/SYSTEM_ARCHITECTURE.md), [ALGORITHMS.md](../02_architecture/ALGORITHMS.md)
