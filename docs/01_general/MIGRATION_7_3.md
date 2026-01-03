# Migration Guide: Phase 7.3 (Hybrid Backend)

## Overview

In version 0.1.0 (Phase 7.3), CueDeck introduces a **Hybrid Backend Architecture**. This significant performance upgrade combines the simplicity of JSON for document storage with the speed of SQLite for metadata querying.

**Benefits:**

- **24x Faster Token Counting:** Queries aggregated across thousands of files in microseconds.
- **Improved Scalability:** Handle larger workspaces without linear performance degradation during rendering.
- **Atomic Operations:** File updates are now transaction-safe.

## What Changed?

| Feature | Previous (JSON Only) | New (Hybrid) |
| :--- | :--- | :--- |
| **Document Storage** | `documents.bin` (JSON) | `documents.bin` (JSON) |
| **Metadata Index** | In-memory Scan | `metadata.db` (SQLite) |
| **Query Speed** | O(n) - Linear Scan | O(1) / O(log n) - Indexed SQL |
| **Migration** | N/A | Automatic on startup |

## Automatic Migration

No manual action is required. When you launch CueDeck (or use the CLI), the system detects if you are on the old version and performs the following steps automatically:

1. **Locking:** Acquires an exclusive lock (`.cue/migration.lock`) to prevent concurrent migrations.
2. **Backup:** Creates a backup of your existing `documents.bin` to `.cue/documents.bin.backup.<timestamp>`.
3. **Initialization:** Creates a new `.cue/metadata.db` SQLite database.
4. **Population:** Reads all documents from the JSON cache and inserts their metadata (path, hash, size, tokens) into the database.
5. **Completion:** migration lock is released.

> [!NOTE]
> Migration typically takes less than **100ms** for workspaces with hundreds of files.

## Troubleshooting

### Migration Stuck?

If CueDeck crashes during migration, it might leave a `migration.lock` file.
**Fix:**

1. Ensure no CueDeck processes are running.
2. Delete `.cue/migration.lock`.
3. Restart CueDeck.

### Database Corruption?

If the migration fails repeatedly, a `migration_failed.marker` file is created to prevent boot loops.
**Fix:**

1. Check logs for the specific error.
2. Delete `.cue/metadata.db` and `.cue/migration_failed.marker`.
3. Restart CueDeck to force a fresh migration.

### Rollback

To revert to the pure JSON state (effectively uninstalling the DB index):

1. Stop CueDeck.
2. Delete `.cue/metadata.db`, `.cue/metadata.db-wal`, and `.cue/metadata.db-shm`.
3. CueDeck will regenerate the DB on next run (or you can downgrade usage if supported).

## For Developers

The `CueEngine` now initializes a `DbManager`. To access raw SQL power:

```rust
let db = engine.db.as_ref().unwrap();
let count = db.get_total_tokens()?;
```

See `crates/cue_core/src/db/mod.rs` for the full API.
