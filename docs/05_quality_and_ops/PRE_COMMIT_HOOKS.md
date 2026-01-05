# Pre-commit Hooks

CueDeck provides pre-commit hooks to validate workspace integrity before commits. This ensures AI agents always receive valid, well-structured task data.

## Installation

### Option 1: Manual Copy

```bash
cp .cuedeck/hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Option 2: Symlink (Recommended for Teams)

```bash
ln -sf ../../.cuedeck/hooks/pre-commit .git/hooks/pre-commit
```

## What It Checks

The pre-commit hook runs `cue doctor --strict` which validates:

| Check | Description | Severity |
|:------|:------------|:---------|
| Config File | Valid TOML syntax | Warn/Fail |
| Workspace Structure | Required directories exist | Fail |
| Card Frontmatter | Valid YAML, required fields | Fail |
| Link Integrity | All references resolve | Fail |
| Metadata Consistency | Timestamps, formats | Warn |
| Task Dependencies | No cycles, all IDs exist | Fail |
| Orphaned Tasks | Tasks with no connections | Warn |
| Missing Dependencies | Invalid `depends_on` refs | Fail |

## Strict Mode

In strict mode (`--strict` or `--fail-on-warnings`), warnings are treated as errors. This ensures the highest data quality for AI agents.

## Bypassing the Hook

For emergencies, you can bypass the hook:

```bash
git commit --no-verify -m "Emergency fix"
```

> [!CAUTION]
> Bypassing validation may introduce invalid data that breaks AI agent workflows.

## Auto-fix Issues

Many issues can be automatically fixed:

```bash
cue doctor --repair
```

---
**Related Docs**: [CLI_REFERENCE.md](../04_tools_and_data/CLI_REFERENCE.md), [TESTING_STRATEGY.md](./TESTING_STRATEGY.md)
