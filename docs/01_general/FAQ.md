# Frequently Asked Questions (FAQ)

## General

### What is CueDeck?

CueDeck is a context management tool for AI-assisted development. It generates smart, pruned snapshots of your project to fit within LLM token limits.

### Who is this for?

- Developers using AI coding assistants (Cursor, GitHub Copilot, etc.)
- Teams managing complex codebases
- Anyone who needs intelligent context selection

### Why Rust?

- **Performance**: Sub-5ms incremental updates
- **Safety**: No runtime errors from memory issues
- **Ecosystem**: Excellent CLI and parsing libraries

## Technical

### Why not use a vector database?

Codebases are **structured graphs**, not semantic soup. We use:

- Filesystem hierarchy (natural organization)
- Explicit references (`refs:` in frontmatter)
- Topological sorting (deterministic, fast)

This gives you **predictable, auditable** results.

### How does token pruning work?

We use a **greedy knapsack** algorithm:

1. Always include the active card (root)
2. Include direct references (depth 1)
3. Fill remaining space with depth 2+ by priority

See [ALGORITHMS.md](../02_architecture/ALGORITHMS.md#2-token-pruning-the-knapsack-lite) for details.

### How are secrets protected?

The **Secret Guard** runs as a final filter:

- Regex-based pattern matching
- Default patterns for AWS, GitHub, OpenAI keys
- Customizable via `config.toml`

See [SECURITY.md](../02_architecture/SECURITY.md) for implementation.

### Can I extend CueDeck with plugins?

Not in v2.1.0. The architecture is designed to be extended via:

- Custom MCP tools (you can add your own)
- Configuration-based rules (`.cuedeck/security.rules`)

Plugin system is on the [ROADMAP.md](./ROADMAP.md) for Phase 5.

## Usage

### What's the difference between `cards/` and `docs/`?

| Directory | Purpose             | Lifecycle                   |
| :-------- | :------------------ | :-------------------------- |
| `cards/`  | Ephemeral tasks     | Created → Done → Archived   |
| `docs/`   | Long-term knowledge | Permanent, slowly evolving  |

### How do I handle large files?

Use **anchor references**:

```yaml
refs:
  - "docs/api.md#Authentication"  # ✅ Only this section
  - "docs/api.md"                 # ❌ Entire file
```

### Why is my scene empty?

Check:

1. Are cards marked as `active`?
2. Run `cue doctor` to check for config issues
3. Verify `.cuedeckignore` isn't blocking files

### How do I reset everything?

```bash
cue clean  # Removes .cuedeck/.cache
```

## MCP Integration

### How do I connect to my AI tool?

Add CueDeck as an MCP server in your tool's config:

**Cursor** (`~/.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "cuedeck": {
      "command": "cue",
      "args": ["mcp"]
    }
  }
}
```

### What tools are available?

- `read_context`: Fuzzy search files
- `read_doc`: Get file/section content
- `list_tasks`: Filter cards by status
- `update_task`: Modify card frontmatter

See [TOOLS_SPEC.md](../04_tools_and_data/TOOLS_SPEC.md) for schemas.

## Development

### How do I contribute?

1. Read [CONTRIBUTING.md](./CONTRIBUTING.md)
2. Check [ROADMAP.md](./ROADMAP.md) for priorities
3. Start with issues tagged `good-first-issue`

### What's the workspace architecture?

```
cue_cli → cue_core → cue_config → cue_common
            ↓
         cue_mcp
```

See [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md) for details.

### Where do I start coding?

1. Follow [QUICK_START.md](./QUICK_START.md) to build
2. Check [IMPLEMENTATION_PATTERNS.md](../02_architecture/IMPLEMENTATION_PATTERNS.md)
3. Run `cargo test` to verify setup

## Diagnostic Commands

### Quick Health Check

```bash
# Check workspace health
cue doctor --verbose

# Verify cache integrity
cue clean && cue scene --dry-run

# List all active tasks
cue list --status=active

# Show cache statistics
cue doctor | grep "Cache:"
```

### Common Diagnostics

| Issue | Command | Expected Output |
| :--- | :--- | :--- |
| **Cache stale** | `cue clean` | `✓ Cache rebuilt` |
| **Circular refs** | `cue doctor` | Error 1002 + cycle path |
| **Token limit** | `cue scene --limit=50000` | Success with higher limit |
| **Missing files** | `cue doctor --repair --issue=dead-links` | `✓ Repaired N cards` |
| **Zombie entries** | `cue doctor --repair --issue=orphan-cards` | `✓ Removed M zombies` |

### Performance Profiling

```bash
# Benchmark scene generation
time cue scene > /dev/null

# Check parser performance
RUST_LOG=debug cue scene 2>&1 | grep "Parser"

# Monitor file watcher
cue watch --verbose &
touch test.md  # Should trigger update within 600ms
```

## Troubleshooting

### "Cycle Detected" error?

Remove conflicting `refs:` entries. See [TROUBLESHOOTING.md](../05_quality_and_ops/TROUBLESHOOTING.md#cycle-detected-error-1002).

### File changes not detected by watcher?

Check ignore patterns and increase `fs.inotify.max_user_watches` on Linux.

### Token limit exceeded?

1. Increase `token_limit` in config
2. Use anchor references
3. Archive old cards

---
**Related Docs**: [GLOSSARY.md](./GLOSSARY.md), [TROUBLESHOOTING.md](../05_quality_and_ops/TROUBLESHOOTING.md)
