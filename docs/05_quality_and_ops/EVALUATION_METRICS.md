# Evaluation Metrics

## 1. Performance Baselines (Bench via `criterion`)

| Metric | Target | Failure Threshold |
| :--- | :--- | :--- |
| **Hot Path** (Inc. Update) | **< 5ms** | > 20ms |
| **Cold Start** (100 files) | **< 1s** | > 3s |
| **Memory Footprint** | **< 50MB** | > 200MB |

## 2. Reliability KPIs

### Cache Hit Rate

**Formula**:

```text
Cache Hit Rate = (Cache Hits / Total Reads) × 100%
```

**Target**: > 95%  
**Measurement**: Track in `CacheManager` telemetry  
**Calculation Example**:

```text
Session: 100 file reads
Cache hits: 97
Cache misses: 3
Hit Rate = (97 / 100) × 100% = 97% ✅
```

### Cache Rot (Staleness)

**Formula**:

```text
Cache Rot Rate = (Stale Entries / Total Cached Entries) × 100%
```

**Target**: 0%  
**Validation**: `hash(file_content) == cached_metadata.hash`  
**Detection**: On every cache read, verify hash matches

### Cycle Rejection

**Formula**:

```text
Cycle Detection Success = (Detected Cycles / Actual Cycles) × 100%
```

**Target**: 100%  
**Test Case**: Introduce `A → B → C → A` in test suite  
**Expected**: Immediate `CyclicDependency` error (code 1002)

## 3. System Maintenance Metrics

| Metric | Target | Failure Threshold |
| :--- | :--- | :--- |
| **Watcher Latency** | **< 500ms** (Debounce) | > 2s |
| **Upgrade Success Rate** | **100%** | Any failure (Check Checksum) |

## 4. Agent Quality

### Context Density

**Formula**:

```text
Context Density = (Relevant Lines / Total Lines in SCENE.md) × 100%
```

**Target**: > 80%  
**Measurement**: Manual review of 10 random scenes per week  
**Optimization**: Adjust pruning priority in `cue_core::graph::priority_score()`

### Hallucination Rate

**Formula**:

```text
Hallucination Rate = (Invalid File Refs / Total File Refs) × 100%
```

**Target**: < 1%  
**Detection**: Parse agent responses for `@ref` patterns, validate against file index  
**Example**:

```text
Agent mentions: "See @ref[docs/missing.md]"
Validation: File does not exist → Hallucination ❌
```

## 4. Token Efficiency (Extended)

### Baseline & Targets

| Metric | Baseline | Target | Measurement |
| :--- | :--- | :--- | :--- |
| **Avg tokens per session** | Week 1 measurement | **-40%** by Week 6 | Per workflow type |
| **Context loading time** | - | **< 1s** | Benchmark |
| **Memory usage** | - | **< 500MB** | Runtime profiling |
| **Rule matching speed** | - | **< 100ms per file** | Benchmark |

### Token Tracking

> [!NOTE]
> **Token budgets are defined in [GOVERNANCE_TEMPLATES.md](../03_agent_design/GOVERNANCE_TEMPLATES.md#canonical-token-budgets-single-source-of-truth)**
>
> Default budgets: Feature Development (6000), Bug Fix (4000), Refactoring (5000), Review (3000)

For detailed breakdown and recalibration policy, see the canonical source above.

## 5. Context Accuracy

**Formula**:

```text
Context Accuracy = (Correct Context Retrievals / Total Retrievals) × 100%
```

**Target**: ≥ 95%  
**Measurement Protocol**:

1. Sample 20 agent interactions per week
2. Manually verify retrieved context matches user intent
3. Track "forgetting events" (agent asks for info already provided)
4. Count incorrect assumptions made by agent

**Example Evaluation**:

```text
Week 4: 20 interactions sampled
- Correct context: 19
- Incorrect/Missing: 1
Accuracy = (19 / 20) × 100% = 95% ✅
```

## 6. Productivity Tracking

**Formula**:

```text
Productivity Gain = ((Baseline Time - Agent Time) / Baseline Time) × 100%
```

**Target**: 60% faster  
**Measurement**:

- **Baseline**: Track 10 tasks completed manually (without CueDeck)
- **Agent-Assisted**: Track same 10 task types with CueDeck + Agent
- **Calculate**: Average time savings

**Example**:

```text
Task: "Add authentication to API endpoint"
Baseline (Manual): 120 minutes
With Agent: 45 minutes
Gain = ((120 - 45) / 120) × 100% = 62.5% ✅
```

## 7. Measurement Cadence

```text
Week 2, 4, 6, 8, 10:
- Review metrics
- Identify bottlenecks
- Adjust token budgets
- Optimize slow paths
- Update documentation
```

## 8. Success Criteria Matrix

| Phase | Metric | Target | Measurement | Pass/Fail |
| :--- | :--- | :--- | :--- | :--- |
| **Foundation** | Hot Path | < 5ms | `cargo bench` | ✅ / ❌ |
| **Foundation** | Cold Start | < 1s | `criterion` | ✅ / ❌ |
| **Core Brain** | Cache Hit Rate | > 95% | Telemetry | ✅ / ❌ |
| **Core Brain** | Cycle Detection | 100% | Unit tests | ✅ / ❌ |
| **CLI** | Watcher Latency | < 500ms | Manual test | ✅ / ❌ |
| **MCP** | Context Density | > 80% | Manual review | ✅ / ❌ |
| **MCP** | Hallucination Rate | < 1% | Validation script | ✅ / ❌ |
| **Polish** | Productivity Gain | 60% | Time tracking | ✅ / ❌ |

### Gate Criteria for Each Phase

**Phase Exit Requirements**:

- **All** ✅ metrics must pass before moving to next phase
- **Any** ❌ metric blocks phase completion
- **Regression**: If any previously passing metric fails, return to that phase

---
**Related Docs**: [TESTING_STRATEGY.md](./TESTING_STRATEGY.md), [SYSTEM_ARCHITECTURE.md](../02_architecture/SYSTEM_ARCHITECTURE.md), [ALGORITHMS.md](../02_architecture/ALGORITHMS.md), [ROADMAP.md](../../01_general/ROADMAP.md)
