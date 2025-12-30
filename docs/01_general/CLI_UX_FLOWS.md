# CLI UX Flows

## 1. `cue open` (Fuzzy Finder)

**Goal**: Instant navigation.

### Visual State (Skim)

```text
> [Query Input]        (Cursor blinking)
  12/50            (Items matched / Total)
------------------------------------------
> docs/01_general/USER_STORIES.md  (Cyan)
  crates/cue_core/src/parser.rs    (White)
  .cuedeck/cards/2a9f1x.md         (Green - Active Card)
```

### Interactions

- **Typing**: Filters list instantly.
- **Colors**:
  - **Cards**: Green (Active), Gray (Archived).
  - **Docs**: Cyan.
  - **Code**: White.
- **Enter**: Opens `$EDITOR` (VSCode/Neovim) at the file path.

## 2. `cue scene` (Context Gen)

**Goal**: Feedback on success/failure.

### Success Output (stderr)

```text
[INFO]  Workspace loaded (150ms)
[INFO]  Parsed 12 files (4 changed)
[CHECK] Token Budget: 12,450 / 32,000 (OK)
[DONE]  Scene copied to clipboard! (Size: 45KB)
```

### Warning Output

```text
[WARN]  Token limit exceeded (35,000 > 32,000).
[WARN]  Pruned 4 low-priority files from context.
```

## 3. `cue doctor` (Health Check)

**Goal**: Confidence in environment.

### Output

```text
Running CueDeck Doctor...

[OK] Workspace Root: .cuedeck/ found.
[OK] Config: Valid TOML.
[ERR] Dead Link: 'docs/auth.md' references 'docs/missing-file.md'.
[OK] Cache: Healthy (45 entries).

Result: 1 Issue Found.
```

**Exit Code**: 1

---
**Related Docs**: [CLI_REFERENCE.md](../04_tools_and_data/CLI_REFERENCE.md), [USER_STORIES.md](./USER_STORIES.md)
