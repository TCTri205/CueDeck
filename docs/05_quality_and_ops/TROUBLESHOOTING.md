# Troubleshooting Guide

## 1. Common Issues

### "CueDeck not detecting changes"

- **Symptom**: You save a file, but `cue watch` doesn't update clipboard.
- **Cause**:
    1. File is in `.gitignore` or `.cuedeckignore`.
    2. `notify` watcher limits reached (common on Linux).
- **Fix**: Check `config.toml` ignore patterns. On Linux, increase `fs.inotify.max_user_watches`.

### "Cycle Detected Error (1002)"

- **Symptom**: `cue scene` fails with `A -> B -> A`.
- **Fix**: Open File A or B and remove the conflicting `refs:` entry in the frontmatter.

### "Token Limit Exceeded (1003)"

- **Symptom**: `SCENE.md` is cut off.
- **Fix**:
    1. Increase `token_limit` in `.cuedeck/config.toml`.
    2. Use more granular references (e.g., link to `#Header` instead of full file).
    3. Archive old tasks in `.cuedeck/cards` (move status to `archived`).

## 2. The `cue doctor` Command

Run this command to auto-diagnose issues.

| Check | Description |
| :--- | :--- |
| **Workspace Root** | Verifies `.cuedeck/` exists. |
| **Config Validity** | Parses `config.toml` for syntax errors. |
| **Dead Links** | Scans all `refs:` for paths that don't exist. |
| **Orphan Cards** | Warns about `active` cards assigned to nobody. |
| **Cache Health** | Verifies `metadata.json` integrity. |

## 3. Resetting State

If the app behaves strangely (e.g., stale content), force a clean slate:

```bash
cue clean
# output: Removed .cuedeck/.cache
```

Then run the command again. It will rebuild the cache from scratch (Cold Start).

---
**Related Docs**: [CLI_REFERENCE.md](../04_tools_and_data/CLI_REFERENCE.md), [SECURITY.md](../02_architecture/SECURITY.md), [CONTRIBUTING.md](../01_general/CONTRIBUTING.md)
