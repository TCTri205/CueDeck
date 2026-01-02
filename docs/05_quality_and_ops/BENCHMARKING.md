# Performance Benchmarking Guide

This guide explains how to run, interpret, and maintain performance benchmarks for CueDeck.

## Overview

CueDeck uses [Criterion.rs](https://github.com/bheisler/criterion.rs) for performance benchmarking. Benchmarks establish baseline metrics and detect performance regressions.

## Running Benchmarks

### All Benchmarks

```bash
cargo bench --package cue_core
```

### Specific Benchmark

```bash
# Only parsing benchmarks
cargo bench --package cue_core -- parsing

# Only search benchmarks
cargo bench --package cue_core -- search

# Verbose output
cargo bench --package cue_core -- --verbose
```

### With HTML Reports

Criterion generates HTML reports in `target/criterion/`:

```bash
cargo bench --package cue_core
open target/criterion/report/index.html  # macOS/Linux
start target/criterion/report/index.html # Windows
```

## Benchmark Suite

### Parsing Benchmarks

Tests file parsing performance:

- **parse_10_files**: Parse 10 markdown files
- **parse_100_files**: Parse 100 markdown files

**Purpose**: Measure parser overhead and scalability.

### Search Benchmarks

Tests search performance across all modes:

- **keyword_search**: Fast text matching (no embeddings)
- **semantic_search_warm**: AI similarity search (cached embeddings)
- **hybrid_search**: Combined keyword + semantic (70/30 weighting)

**Purpose**: Measure search modes and cache effectiveness.

## Baseline Metrics (v2.2.0)

Established on 2026-01-02 using:

- **Hardware**: Windows 11, 8-core CPU
- **Dataset**: 100 markdown files (~300 tokens each)
- **Criterion**: v0.5.1
- **Profile**: `--release` (optimized)

| Benchmark | Time (mean) | Std Dev | Outliers |
|:---|:---|:---|:---|
| **Parsing** | | | |
| parse_10_files | 8.0 ms | ±0.39 ms | 4% |
| parse_100_files | 91.2 ms | ±19.0 ms | 8% |
| **Search** | | | |
| keyword_search | 90.8 ms | ±5.8 ms | 3% |
| semantic_search_warm | **47.4 ms** | ±10.3 ms | 5% |
| hybrid_search | 49.5 ms | ±6.2 ms | 2% |

### Key Insights

1. **Linear Parsing Scaling**: ~800-900 µs per file (consistent)
2. **Semantic Faster Than Keyword**: 2x faster due to embedding cache
3. **Hybrid Close to Semantic**: Only ~2ms overhead for combined scoring
4. **Cache Effectiveness**: Warm semantic search is production-ready

## Interpreting Results

### Understanding Criterion Output

```
parsing/parse_100_files time:   [88.900 ms 91.209 ms 94.032 ms]
                                 ^^^^^^^^^  ^^^^^^^^^  ^^^^^^^^^
                                 Lower      Mean       Upper
                                 bound                 bound
```

- **Mean**: Average execution time
- **Bounds**: 95% confidence interval
- **Outliers**: Measurements significantly different from mean

### What to Look For

✅ **Good Signs**:

- Low standard deviation (< 20%)
- Few outliers (< 5%)
- Consistent mean across runs

⚠️ **Warning Signs**:

- High std dev (> 30%) - indicates instability
- Many outliers - indicates external interference
- Mean drift over time - potential regression

## Regression Detection

### Manual Comparison

```bash
# Run baseline
cargo bench --package cue_core -- --save-baseline v2.2.0

# After changes, compare
cargo bench --package cue_core -- --baseline v2.2.0
```

Criterion will show % change:

```
parsing/parse_100_files time:   [91.2 ms 93.5 ms 95.8 ms]
                        change: [+2.51% +5.10% +7.89%] (p = 0.00 < 0.05)
                        Performance has REGRESSED.
```

### Acceptable Thresholds

- **< 5% change**: Normal variance, accept
- **5-10% slower**: Review, may accept if justified
- **> 10% slower**: **Regression**, investigate required
- **> 10% faster**: Verify correctness, then celebrate! 🎉

## CI Integration

Benchmarks run in CI on every PR to `main`:

```yaml
# .github/workflows/benchmarks.yml
- name: Run benchmarks
  run: cargo bench --package cue_core -- --save-baseline pr-${{github.event.number}}
  
- name: Compare with main
  run: |
    git fetch origin main
    git checkout main
    cargo bench --package cue_core -- --save-baseline main
    git checkout -
    cargo bench --package cue_core -- --baseline main
```

## Best Practices

### Before Benchmarking

1. **Close Background Apps**: Minimize noise
2. **Disable Power Saving**: Ensure consistent CPU frequency
3. **Warm Up System**: Run once before measuring
4. **Consistent State**: Same dataset, no other processes

### Writing Benchmarks

```rust
use criterion::{black_box, Criterion};

fn bench_something(c: &mut Criterion) {
    let setup_data = prepare_data();
    
    c.bench_function("my_operation", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations
            my_operation(black_box(&setup_data))
        });
    });
}
```

**Key Tips**:

- Use `black_box()` to prevent dead code elimination
- Setup outside the iterator (don't measure setup)
- Use `&mut` only if testing mutation
- Keep benchmarks focused (one thing at a time)

### Debugging Slow Benchmarks

1. **Profile with cargo-flamegraph**:

   ```bash
   cargo install flamegraph
   cargo flamegraph --bench performance -- --bench keyword_search
   ```

2. **Check allocation patterns**:

   ```bash
   cargo bench --features dhat-heap
   ```

3. **Review hot paths**:
   - Look for unexpected allocations
   - Check for redundant clones
   - Verify cache effectiveness

## Test Data

Benchmarks use generated test data in `benches/test_data/`:

- **100 markdown files** with frontmatter
- **Varied content**: Tags, priorities, sample text
- **Consistent structure**: Enables reproducible benchmarks

Data is auto-generated on first run.

## Troubleshooting

### "Gnuplot not found"

Criterion uses Plotters backend as fallback. To get plots:

```bash
# Windows (Chocolatey)
choco install gnuplot

# macOS
brew install gnuplot

# Linux
sudo apt-get install gnuplot
```

### Benchmark Takes Too Long

Reduce sample count:

```bash
cargo bench -- --sample-size 50
```

Or reduce warmup time:

```bash
cargo bench -- --warm-up-time 1
```

### Inconsistent Results

Common causes:

- Background processes consuming CPU
- Power management throttling
- Thermal throttling (check cooling)
- Insufficient dataset size

## Future Enhancements

- [ ] Add memory usage benchmarks
- [ ] Benchmark with larger datasets (500/1000 files)
- [ ] Add cold cache benchmarks (no embedding cache)
- [ ] Benchmark graph resolution  
- [ ] Add benchmarks for MCP tools

---

**Related Docs**:

- [PERFORMANCE_OPTIMIZATION.md](../02_architecture/PERFORMANCE_OPTIMIZATION.md)
- [ALGORITHMS.md](../02_architecture/ALGORITHMS.md#7-hybrid-search-algorithm)
