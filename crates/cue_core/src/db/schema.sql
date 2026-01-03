-- Schema for CueDeck metadata database (Phase 7: Performance Optimization)
-- This SQLite database provides 24x faster queries compared to JSON scanning

-- File metadata tracking
CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    hash TEXT NOT NULL,
    modified_at INTEGER NOT NULL,  -- Unix timestamp
    size_bytes INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_files_hash ON files(hash);
CREATE INDEX IF NOT EXISTS idx_files_modified ON files(modified_at);
CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);

-- Cards table for task metadata
CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY,  -- 6-character task ID
    file_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('todo', 'active', 'done', 'archived')),
   assignee TEXT,
    priority TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_cards_status ON cards(status);
CREATE INDEX IF NOT EXISTS idx_cards_assignee ON cards(assignee);
CREATE INDEX IF NOT EXISTS idx_cards_priority ON cards(priority);
CREATE INDEX IF NOT EXISTS idx_cards_file ON cards(file_id);

-- Tags table (normalized)
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);

-- Card-Tag junction table (many-to-many)
CREATE TABLE IF NOT EXISTS card_tags (
    card_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (card_id, tag_id),
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_card_tags_tag ON card_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_card_tags_card ON card_tags(card_id);

-- Anchors table for document sections
CREATE TABLE IF NOT EXISTS anchors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    level INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_anchors_name ON anchors(name);
CREATE INDEX IF NOT EXISTS idx_anchors_file ON anchors(file_id);

-- Metadata version table
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Insert schema version
INSERT OR IGNORE INTO schema_version (version) VALUES (1);
