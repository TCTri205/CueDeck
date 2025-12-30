# Knowledge Base Structure

## 1. Directory Layout (The "Workspace")

```text
.cuedeck/
├── cards/                   # Ephemeral Task Cards
│   ├── 2a9f1x.md           # ID = 2a9f1x (Short)
│   └── 8b2c4z.md
├── docs/                    # Long-lived Documentation
│   ├── api.md
│   └── architecture.md
├── .cache/                  # Git-ignored Runtime State
│   ├── metadata.json       # The Brain State
│   └── logs/               # Application Logs
└── config.toml              # Local Overrides
```

## 2. Data Schemas

### A. Card Frontmatter (YAML)

**File**: `.cuedeck/cards/2a9f1x.md`

```yaml
---
id: "2a9f1x"              # REQUIRED: 6-char alphanumeric hash
uuid: "550e..."           # SYSTEM: Full V4 UUID for tracking
title: "Implement Login"  # REQUIRED: Human readable
status: "active"          # ENUM: [todo, active, done, archived]
priority: "high"          # ENUM: [low, medium, high, critical]
assignee: "user"          # OPTIONAL
refs:                     # OPTIONAL: Explicit Graph Edges
  - "docs/auth.md#Flow"   # -> Points to specific anchor
  - "crates/cue_core"     # -> Points to directory
---
```

### B. Cache Metadata (JSON)

**File**: `.cuedeck/.cache/metadata.json`

```json
{
  "version": "2.1",
  "files": {
    "docs/api.md": {
      "hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "last_checked_ts": 1709251200,
      "token_count": 1420,
      "anchors": [
        "Authentication",
        "Authentication > Rate Limiting"
      ]
    }
  }
}
```

**Key Behavior**:

- If `files[path].hash != sha256(disk_content)`, the entry is stale.
- If `path` does not exist on disk, the entry is a "Zombie" (removed on next GC).

### C. Configuration Reference (`config.toml`)

**File**: `.cuedeck/config.toml` (or `~/.config/cuedeck/config.toml`)

### C. Configuration Reference

**File**: `.cuedeck/config.toml`

> **Note**: For the full list of options, defaults, and environment variable overrides, see [`CONFIGURATION_REFERENCE.md`](./CONFIGURATION_REFERENCE.md).

```toml
[core]
token_limit = 32000
```

---
**Related Docs**: [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md), [SECURITY.md](../02_architecture/SECURITY.md), [GLOSSARY.md](../01_general/GLOSSARY.md)
