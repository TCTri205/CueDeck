# CueDeck Maintenance Guide

This document provides operational procedures for maintaining a healthy CueDeck installation.

## Regular Maintenance Tasks

### Daily Tasks (Automated)

**Cache Health Check** (automatically runs during `cue scene`):

```rust
// Runs internally on each operation
fn check_cache_health() {
    - Verify metadata.json is well-formed
    - Check for orphaned entries (files deleted from disk)
    - Validate hash consistency
}
```

**No user action required** - cache self-heals automatically.

### Weekly Tasks

**1. Cache Statistics Review**

```bash
# Check cache stats
cue doctor --verbose

# Expected output:
# ✓ Cache status: Healthy
# ✓ Cached files: 247
# ✓ Orphaned entries: 0
# ✓ Cache size: 2.3 MB
# ✓ Hit rate (last 7 days): 97.2%
```

#### Action Required If

- Hit rate < 85%: Consider increasing cache retention
- Orphaned entries > 10%: Run `cue clean`
- Cache size > 50MB: Investigate large files

**2. Log Rotation**

```bash
# Check log size
du -h .cuedeck/logs/

# Rotate if > 100MB
mv .cuedeck/logs/mcp.log .cuedeck/logs/mcp.log.1
gzip .cuedeck/logs/mcp.log.1

# Delete old logs (optional)
find .cuedeck/logs/ -name "*.gz" -mtime +30 -delete
```

### Monthly Tasks

**1. Dependency Updates**

```bash
# Check for outdated dependencies
cargo outdated

# Update Cargo.lock (safe)
cargo update

# Run full test suite
cargo test --workspace

# If tests pass, commit
git add Cargo.lock
git commit -m "chore: update dependencies"
```

**2. Performance Benchmark**

```bash
# Run benchmark suite
cargo bench --bench scene_generation

# Compare with baseline
cargo bench --bench scene_generation -- --baseline month_ago

# If performance degraded >10%, investigate
```

**3. Archive Old Cards**

```bash
# List completed cards older than 30 days
cue card list --status done --older-than 30d

# Archive them
cue card archive --older-than 30d

# Moves to .cuedeck/archive/ (excluded from scenes)
```

---

## 2. Cache Cleanup Procedures

### 2.1 Soft Clean (Recommended)

Removes orphaned entries, keeps valid cache:

```bash
cue doctor --fix

# This will:
# - Remove entries for deleted files
# - Verify hash consistency
# - Compact metadata.json
```

### 2.2 Hard Clean (Nuclear Option)

Complete cache rebuild:

```bash
# WARNING: Destroys all cached metadata
cue clean

# Next scene generation will rebuild cache from scratch
# Expect slower performance (one-time cost)
```

#### When to Hard Clean

- Cache corruption detected
- After major file reorganization
- Debugging cache-related issues

### 2.3 Automated Cleanup Script

```bash
#!/bin/bash
# .cuedeck/scripts/cleanup.sh

set -e

echo "Running CueDeck maintenance..."

# 1. Check health
cue doctor || {
    echo "Health check failed. Running repair..."
    cue doctor --fix
}

# 2. Remove orphans
orphan_count=$(cue doctor --json | jq '.orphaned_entries')
if [ "$orphan_count" -gt 10 ]; then
    echo "High orphan count ($orphan_count). Cleaning..."
    cue doctor --fix
fi

# 3. Archive old cards
cue card archive --older-than 30d

echo "Maintenance complete ✓"
```

**Schedule with cron**:

```cron
# Run every Sunday at 2 AM
0 2 * * 0 cd /path/to/project && .cuedeck/scripts/cleanup.sh
```

---

## 3. Performance Tuning

### 3.1 Adjust Token Limit

**Symptom**: Scenes consistently truncated.

**Solution**: Increase token budget in config:

```toml
# .cuedeck/config.toml

[core]
token_limit = 64000  # Doubled from 32K
```

**Trade-off**: Longer generation time, higher LLM costs.

### 3.2 Optimize Watcher Debounce

**Symptom**: Watch mode triggers too frequently during bulk edits.

**Solution**: Increase debounce interval:

```toml
[watcher]
debounce_ms = 1000  # Increased from 500ms
```

**Trade-off**: Slightly slower scene updates.

### 3.3 Enable Aggressive Caching

**Symptom**: Repeated full-file parses slowing down scene generation.

**Solution**: Pin cache in memory:

```toml
[core]
cache_mode = "memory"  # Default: "disk"
```

**Trade-off**: +20MB memory usage.

---

## 4. Health Check Automation

### 4.1 Integrate with CI/CD

```yaml
# .github/workflows/health-check.yml

name: CueDeck Health Check

on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight
  workflow_dispatch:

jobs:
  health-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install CueDeck
        run: cargo install --path .
      
      - name: Initialize workspace
        run: cue init
      
      - name: Run health check
        run: cue doctor --verbose
      
      - name: Generate test scene
        run: cue scene --dry-run
      
      - name: Check performance
        run: |
          time cue scene --dry-run
          # Fail if >5s
```

### 4.2 Monitoring Script

```bash
#!/bin/bash
# monitor-health.sh

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== CueDeck Health Monitor ==="

# 1. Check cache health
cue doctor --json > /tmp/cue-health.json

hit_rate=$(jq -r '.cache_hit_rate' /tmp/cue-health.json)
if (( $(echo "$hit_rate < 0.85" | bc -l) )); then
    echo -e "${YELLOW}⚠ Low cache hit rate: ${hit_rate}${NC}"
fi

# 2. Check scene generation time
start=$(date +%s%N)
cue scene --dry-run > /dev/null
end=$(date +%s%N)
duration_ms=$(( (end - start) / 1000000 ))

if [ $duration_ms -gt 5000 ]; then
    echo -e "${RED}✗ Scene generation too slow: ${duration_ms}ms${NC}"
elif [ $duration_ms -gt 1000 ]; then
    echo -e "${YELLOW}⚠ Scene generation slow: ${duration_ms}ms${NC}"
else
    echo -e "${GREEN}✓ Scene generation OK: ${duration_ms}ms${NC}"
fi

# 3. Check memory usage
mem_mb=$(ps aux | grep '[c]ue' | awk '{print $6/1024}')
if (( $(echo "$mem_mb > 200" | bc -l) )); then
    echo -e "${RED}✗ High memory usage: ${mem_mb}MB${NC}"
elif (( $(echo "$mem_mb > 100" | bc -l) )); then
    echo -e "${YELLOW}⚠ Elevated memory usage: ${mem_mb}MB${NC}"
else
    echo -e "${GREEN}✓ Memory usage OK: ${mem_mb}MB${NC}"
fi
```

---

## 5. Backup and Disaster Recovery

### 5.1 What to Backup

**Critical** (must backup):

- `.cuedeck/cards/` - Active task cards
- `.cuedeck/docs/` - Documentation
- `.cuedeck/config.toml` - Local configuration

**Optional** (can regenerate):

- `.cuedeck/.cache/` - Metadata cache
- `.cuedeck/SCENE.md` - Current scene snapshot

**Never backup**:

- `.cuedeck/logs/` - Temporary logs

### 5.2 Backup Script

```bash
#!/bin/bash
# backup-cuedeck.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="$HOME/.cuedeck-backups"
BACKUP_FILE="$BACKUP_DIR/cuedeck-backup-$TIMESTAMP.tar.gz"

mkdir -p "$BACKUP_DIR"

echo "Creating CueDeck backup..."

tar -czf "$BACKUP_FILE" \
    --exclude=".cuedeck/.cache" \
    --exclude=".cuedeck/logs" \
    --exclude=".cuedeck/SCENE.md" \
    .cuedeck/

echo "Backup created: $BACKUP_FILE"

# Cleanup old backups (keep last 10)
ls -t "$BACKUP_DIR"/cuedeck-backup-*.tar.gz | tail -n +11 | xargs rm -f

echo "Old backups cleaned (kept last 10)"
```

### 5.3 Restore Procedure

```bash
# 1. Extract backup
tar -xzf cuedeck-backup-20250331_120000.tar.gz

# 2. Rebuild cache
cue clean
cue scene --dry-run  # Triggers cache rebuild

# 3. Verify
cue doctor
```

### 5.4 Automated Git Backup

```bash
# .cuedeck/scripts/git-backup.sh

#!/bin/bash
set -e

cd .cuedeck

# Initialize git if not already
if [ ! -d .git ]; then
    git init
    git remote add origin [your-private-repo]
fi

# Backup
git add cards/ docs/ config.toml
git commit -m "Auto-backup $(date +%Y-%m-%d)"
git push origin main

echo "Backup pushed to git ✓"
```

**Schedule with cron**:

```cron
# Backup daily at 11 PM
0 23 * * * cd /path/to/project && .cuedeck/scripts/git-backup.sh
```

---

## 6. Troubleshooting Common Issues

### Issue: "Cache Rot" (Stale Metadata)

**Symptoms**:

- `cue doctor` reports inconsistencies
- Scenes include deleted files
- Hash mismatches

**Solution**:

```bash
# Option 1: Repair
cue doctor --fix

# Option 2: Rebuild
cue clean
```

### Issue: High Memory Usage

**Symptoms**:

- Process using >200MB RAM
- System becomes sluggish

**Diagnosis**:

```bash
# Check which component is consuming memory
valgrind --tool=massif cue scene
```

**Solution**:

```toml
# Reduce in-memory cache
[core]
cache_mode = "disk"  # Use disk instead of memory
memory_limit_mb = 50  # Limit memory usage
```

### Issue: Slow Scene Generation

**Symptoms**:

- `cue scene` takes >5 seconds

**Diagnosis**:

```bash
# Profile with flamegraph
cargo flamegraph --bin cue -- scene

# Check hot spots in flamegraph.svg
```

**Solution**:

- Check if cache is disabled: `cue doctor`
- Reduce token limit: Lower `token_limit` in config
- Archive old cards: `cue card archive --older-than 30d`

### Issue: Watcher Not Detecting Changes

**Symptoms**:

- `cue watch` doesn't update on file save

**Diagnosis**:

```bash
# Check file system events
inotifywait -m .cuedeck/cards/
```

**Solution** (Linux):

```bash
# Increase inotify watches
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

---

## 7. Maintenance Checklist

### Pre-Release Checklist

Before releasing a new version:

- [ ] Run full test suite: `cargo test --workspace`
- [ ] Run benchmarks: `cargo bench`
- [ ] Update CHANGELOG.md
- [ ] Bump version in `Cargo.toml`
- [ ] Tag release: `git tag v2.2.0`
- [ ] Build release binaries: `cargo build --release`
- [ ] Test on clean workspace: `cue init && cue scene`
- [ ] Verify upgrade works: `cue upgrade`

### Post-Deployment Checklist

After deploying new version:

- [ ] Monitor error rates in logs
- [ ] Check performance metrics
- [ ] Verify cache compatibility
- [ ] Test rollback procedure

---

## 8. Emergency Procedures

### Rollback to Previous Version

```bash
# 1. Download previous version
curl -L https://github.com/cuedeck/releases/download/v2.1.0/cue-linux-x64.tar.gz | tar xz

# 2. Replace binary
sudo mv cue /usr/local/bin/cue

# 3. Verify version
cue --version  # Should show v2.1.0

# 4. Rebuild cache if needed
cue clean && cue scene --dry-run
```

### Recover from Corrupted Cache

```bash
# 1. Backup current state
cp .cuedeck/.cache/metadata.json .cuedeck/.cache/metadata.json.corrupt

# 2. Try repair
cue doctor --fix

# 3. If repair fails, nuke cache
rm -f .cuedeck/.cache/metadata.json
cue scene --dry-run  # Rebuilds from scratch

# 4. Verify
cue doctor
```

### Recover from Corrupted Config

```bash
# 1. Backup corrupt config
cp .cuedeck/config.toml .cuedeck/config.toml.corrupt

# 2. Regenerate default config
cue init --force --defaults-only

# 3. Manually merge your customizations from backup

# 4. Validate
cue doctor
```

---

## 9. Monitoring Dashboard (Optional)

### Prometheus Metrics Export

```rust
// Future feature: Export metrics for Prometheus
// .cuedeck/metrics/prometheus.txt

cuedeck_scene_generation_seconds{quantile="0.5"} 0.018
cuedeck_scene_generation_seconds{quantile="0.9"} 0.045
cuedeck_scene_generation_seconds{quantile="0.99"} 0.125

cuedeck_cache_hit_rate 0.972
cuedeck_cached_files_total 247
cuedeck_memory_usage_bytes 36700160
```

### Grafana Dashboard Template

```json
{
  "dashboard": {
    "title": "CueDeck Performance",
    "panels": [
      {
        "title": "Scene Generation Time (p99)",
        "targets": [{
          "expr": "cuedeck_scene_generation_seconds{quantile=\"0.99\"}"
        }]
      },
      {
        "title": "Cache Hit Rate",
        "targets": [{
          "expr": "cuedeck_cache_hit_rate"
        }]
      }
    ]
  }
}
```

---

**Related Docs**: [TROUBLESHOOTING.md](../05_quality_and_ops/TROUBLESHOOTING.md), [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md), [LOGGING_AND_TELEMETRY.md](./LOGGING_AND_TELEMETRY.md), [PERFORMANCE_OPTIMIZATION.md](../02_architecture/PERFORMANCE_OPTIMIZATION.md)
