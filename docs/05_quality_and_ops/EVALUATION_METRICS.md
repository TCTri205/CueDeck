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

---
**Related Docs**: [TESTING_STRATEGY.md](./TESTING_STRATEGY.md), [SYSTEM_ARCHITECTURE.md](../02_architecture/SYSTEM_ARCHITECTURE.md), [ALGORITHMS.md](../02_architecture/ALGORITHMS.md)
