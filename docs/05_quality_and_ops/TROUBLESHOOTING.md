# Troubleshooting Guide

## Quick Diagnostic Flowchart

```text
Issue detected
    │
    ├─→ `cue doctor` passes? 
    │       ├─ YES → Check specific symptoms below
    │       └─ NO  → Fix reported issues first
    │
    ├─→ Recently updated CueDeck?
    │       └─ YES → Run `cue clean` then retry
    │
    └─→ Works in new workspace?
            ├─ YES → Project-specific config issue
            └─ NO  → Installation/binary issue
```

## 1. Common Issues

### "CueDeck not detecting changes"

- **Symptom**: You save a file, but `cue watch` doesn't update clipboard.
- **Cause**:
    1. File is in `.gitignore` or `.cuedeckignore`.
    2. `notify` watcher limits reached (common on Linux).
    3. Watcher is running in wrong directory.
- **Fix**:
  - Check `config.toml` ignore patterns.
  - On Linux, increase `fs.inotify.max_user_watches`:

      ```bash
      echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
      sudo sysctl -p
      ```

  - Verify `cue watch` is run from workspace root.

### "Cycle Detected Error (1002)"

- **Symptom**: `cue scene` fails with `A -> B -> A`.
- **Cause**: Circular reference in `refs:` frontmatter.
- **Diagnosis**:

    ```bash
    cue doctor --verbose
    # Shows: Cycle: cards/task-a.md -> docs/api.md -> cards/task-a.md
    ```

- **Fix**: Open File A or B and remove the conflicting `refs:` entry.

### "Token Limit Exceeded (1003)"

- **Symptom**: `SCENE.md` is cut off or generation fails.
- **Fix**:
    1. Increase `token_limit` in `.cuedeck/config.toml`:

       ```toml
       [core]
       token_limit = 64000  # Double from 32K
       ```

    2. Use more granular references (e.g., link to `#Header` instead of full file).
    3. Archive old tasks: `cue card archive --older-than 30d`.

### "Secret Masking Too Aggressive"

- **Symptom**: Legitimate code is replaced with `***`.
- **Cause**: Secret pattern regex matches too broadly.
- **Fix**: Customize patterns in `config.toml`:

    ```toml
    [security]
    extra_patterns = [
        "sk-[a-zA-Z0-9]{48}",  # OpenAI
        # Remove overly broad patterns
    ]
    ```

### "MCP Server Not Starting"

- **Symptom**: AI tool (Cursor/Claude) can't connect to CueDeck.
- **Diagnosis**:

    ```bash
    echo '{"method":"ping","id":1}' | cue mcp 2>&1
    # Should return: {"id":1,"result":"pong"}
    ```

- **Causes & Fixes**:

    | Cause | Fix |
    | :--- | :--- |
    | Binary not in PATH | Add CueDeck to PATH |
    | Wrong config in AI tool | Check `mcpServers` JSON |
    | Workspace not initialized | Run `cue init` |

### "Stale SCENE.md Content"

- **Symptom**: Changes to cards/docs not reflected in SCENE.
- **Cause**: Cache metadata doesn't match actual file hashes.
- **Diagnosis**:

    ```bash
    cue doctor --verbose
    # Look for: Cache mismatch: file.md (cached hash != current hash)
    ```

- **Fix**:

    ```bash
    cue clean    # Wipe cache
    cue scene    # Rebuild fresh
    ```

### "Config Parse Error"

- **Symptom**: `Error: Invalid TOML in config.toml`.
- **Fix**: Validate TOML syntax:

    ```bash
    cat .cuedeck/config.toml | python -c "import sys,toml;toml.load(sys.stdin)"
    # Or use online TOML validator
    ```

## 2. The `cue doctor` Command

Run this command to auto-diagnose issues.

| Check | Description | Exit Code |
| :--- | :--- | :--- |
| **Workspace Root** | Verifies `.cuedeck/` exists. | 1 |
| **Config Validity** | Parses `config.toml` for syntax errors. | 2 |
| **Dead Links** | Scans all `refs:` for paths that don't exist. | 3 |
| **Orphan Cards** | Warns about `active` cards assigned to nobody. | 4 |
| **Cache Health** | Verifies `metadata.json` integrity. | 5 |
| **Circular Deps** | Detects cycles in dependency graph. | 6 |

### Verbose Output

```bash
cue doctor --verbose
# Output:
# ✓ Workspace: /home/user/project/.cuedeck
# ✓ Config: Valid TOML, 12 settings loaded
# ✓ Dead Links: 0 found
# ⚠ Orphan Cards: 2 cards with status=active but no assignee
# ✓ Cache: 247 entries, 98.2% hit rate
# ✓ Cycles: No circular dependencies
```

### Automated Repair Commands

```bash
# Repair specific issues automatically
cue doctor --repair --issue=stale-cache   # Rebuild cache for stale entries
cue doctor --repair --issue=dead-links    # Remove invalid refs from frontmatter
cue doctor --repair --issue=orphan-cards  # Prompt to assign or archive
cue doctor --repair --all                 # Fix all auto-fixable issues

# Examples
$ cue doctor --repair --issue=dead-links
Found 3 dead links in cards/task-abc123.md
  - docs/missing.md (removed)
  - archive/old.md (removed)
✓ Repaired 2 cards

$ cue doctor --repair --issue=stale-cache
Rebuilding cache for 12 stale entries...
✓ Cache refreshed (12/12 files)
```

## 3. Resetting State

If the app behaves strangely (e.g., stale content), force a clean slate:

```bash
cue clean
# output: Removed .cuedeck/.cache
```

Then run the command again. It will rebuild the cache from scratch (Cold Start).

### Cache Rot Symptoms

- **Symptom**: Stale or incorrect content in `SCENE.md` despite file changes.
- **Cause**: Cache metadata doesn't match actual file hashes (e.g., after git operations, file moves).
- **Fix**: Run `cue clean` to force a Cold Start rebuild.

### Upgrade Issues

- **Symptom**: `cue upgrade` fails or binary doesn't update.
- **Cause**:
    1. No write permission to binary location.
    2. Network connectivity issues.
    3. Running CueDeck while upgrading.
- **Fix**:
    1. Run with elevated permissions (e.g., `sudo cue upgrade` on Unix).
    2. Check internet connection.
    3. Close all CueDeck instances before upgrading.

## 4. Platform-Specific Issues

### Windows

| Issue | Symptom | Fix |
| :--- | :--- | :--- |
| Path too long | Error accessing deep file | Enable LongPaths in registry |
| Anti-virus blocking | Binary flagged | Whitelist `cue.exe` |
| Watcher not working | No clipboard update | Check Windows Defender exclusions |

### Linux

| Issue | Symptom | Fix |
| :--- | :--- | :--- |
| inotify limit | "Too many open files" | Increase `max_user_watches` |
| Clipboard empty | `cue scene` no copy | Install `xclip` or `wl-copy` |
| Permission denied | Can't run binary | `chmod +x cue` |

### macOS

| Issue | Symptom | Fix |
| :--- | :--- | :--- |
| Quarantine flag | "Cannot open application" | `xattr -d com.apple.quarantine cue` |
| Keychain prompt | Password dialog | Grant cue keychain access |

## 5. Debug Mode

Enable verbose logging for deeper diagnosis:

```bash
RUST_LOG=debug cue scene 2> debug.log
```

**Log file location**: `.cuedeck/logs/mcp.log`

### Key Log Patterns to Look For

```text
# Healthy operation
DEBUG cue_core::parser: Parsing file: cards/task.md
DEBUG cue_core::cache: Cache hit for cards/task.md (hash match)

# Problem indicators
WARN cue_core::dag: Potential cycle at cards/task.md
ERROR cue_mcp::router: Failed to serialize response
```

## 6. Full Diagnostic Collection

For comprehensive debugging or when reporting issues, use the automated diagnostic collection script:

### On Windows

```powershell
.\scripts\collect-diagnostics.ps1
```

**What it collects**:

- System information (OS, PowerShell version)
- Rust/Cargo versions
- Workspace structure
- Configuration file
- Log files
- `cue doctor` output

**Output**: `cuedeck-debug-YYYYMMDD_HHMMSS.zip`

### Diagnostic Archive Contents

```text
cuedeck-debug-YYYYMMDD_HHMMSS/
├── system-info.json       # OS, timestamp
├── rust-info.json         # Rust/Cargo versions
├── workspace-structure.json   # .cuedeck/ tree
├── config.toml            # Your config (sanitized)
├── logs/                  # All .log files
│   ├── mcp.log
│   └── watcher.log
└── cue-doctor.txt         # Full diagnostic output
```

### Manual Collection (Alternative)

If the script fails, collect manually:

```bash
# Create output directory
mkdir cuedeck-debug

# System info
rustc --version > cuedeck-debug/rust.txt
cargo --version >> cuedeck-debug/rust.txt

# Diagnostics
cue doctor --verbose > cuedeck-debug/doctor.txt 2>&1

# Config (remove secrets first!)
cp .cuedeck/config.toml cuedeck-debug/

# Logs
cp .cuedeck/logs/*.log cuedeck-debug/

# Create archive
tar -czf cuedeck-debug.tar.gz cuedeck-debug
```

## 7. Getting Help

1. **Check logs**: `.cuedeck/logs/`
2. **Run diagnostics**: `cue doctor --verbose`
3. **Search issues**: [GitHub Issues](https://github.com/your-org/cuedeck/issues)
4. **Report bug**: Include `cue doctor` output + relevant logs

---
**Related Docs**: [CLI_REFERENCE.md](../04_tools_and_data/CLI_REFERENCE.md), [SECURITY.md](../02_architecture/SECURITY.md), [CONTRIBUTING.md](../01_general/CONTRIBUTING.md), [MAINTENANCE_GUIDE.md](./MAINTENANCE_GUIDE.md)
