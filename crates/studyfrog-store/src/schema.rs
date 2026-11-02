//! Schema as data. See `docs/specs/storage-schema.md`.

use crate::migrate::Migration;
use crate::schema_later::{V4, V5, V6, V7, V8, V9, V10, V11, V12};

/// All migrations, oldest first.
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial schema",
        sql: V1,
    },
    Migration {
        version: 2,
        name: "run plan columns",
        sql: V2,
    },
    Migration {
        version: 3,
        name: "run filter, stack identity and settings, sm2 last review",
        sql: V3,
    },
    Migration {
        version: 4,
        name: "sessions and board",
        sql: V4,
    },
    Migration {
        version: 5,
        name: "achievement unlocks",
        sql: V5,
    },
    Migration {
        version: 6,
        name: "answer scores",
        sql: V6,
    },
    Migration {
        version: 7,
        name: "run shuffle switch",
        sql: V7,
    },
    Migration {
        version: 8,
        name: "calendar entry kinds and reminders",
        sql: V8,
    },
    Migration {
        version: 9,
        name: "item priority and difficulty",
        sql: V9,
    },
    Migration {
        version: 10,
        name: "stack subjects",
        sql: V10,
    },
    Migration {
        version: 11,
        name: "pdf documents and page text",
        sql: V11,
    },
    Migration {
        version: 12,
        name: "item status",
        sql: V12,
    },
];

/// The run filter (list-valued parts in their own tables, with a position to keep their order),
/// per-stack identity and settings (derived index, each following its own file), and the
/// last-review time SM-2 items need for queue building.
const V3: &str = "
ALTER TABLE run ADD COLUMN filter_state TEXT NOT NULL DEFAULT 'any'
    CHECK (filter_state IN ('any', 'new', 'learned', 'due', 'not_due'));
ALTER TABLE run ADD COLUMN filter_diff_min REAL;
ALTER TABLE run ADD COLUMN filter_diff_max REAL;

CREATE TABLE run_filter_kind (
    run_uid TEXT NOT NULL,
    pos     INTEGER NOT NULL,
    kind    TEXT NOT NULL CHECK (kind IN ('note', 'card', 'question')),
    PRIMARY KEY (run_uid, pos)
) WITHOUT ROWID;
CREATE TABLE run_filter_stack (
    run_uid   TEXT NOT NULL,
    pos       INTEGER NOT NULL,
    stack_uid TEXT NOT NULL,
    PRIMARY KEY (run_uid, pos)
) WITHOUT ROWID;
CREATE TABLE run_filter_tag (
    run_uid TEXT NOT NULL,
    pos     INTEGER NOT NULL,
    tag     TEXT NOT NULL,
    PRIMARY KEY (run_uid, pos)
) WITHOUT ROWID;

-- Identity comes from the `_stack.md` file, settings from the sibling `_stack.toml`; each follows its
-- own file, and the view joins them by folder.
CREATE TABLE stack_identity (
    path   TEXT PRIMARY KEY REFERENCES file(path) ON DELETE CASCADE ON UPDATE CASCADE,
    uid    TEXT NOT NULL,
    folder TEXT NOT NULL UNIQUE
) WITHOUT ROWID;

CREATE TABLE stack_config (
    folder             TEXT PRIMARY KEY,
    size               INTEGER NOT NULL,
    mtime_ns           INTEGER NOT NULL,
    scheduler          TEXT CHECK (scheduler IN ('fsrs', 'sm2')),
    request_retention  REAL,
    daily_new_limit    INTEGER,
    daily_review_limit INTEGER,
    error              TEXT
) WITHOUT ROWID;

CREATE VIEW stack_setting AS
    SELECT i.uid AS uid, i.folder AS folder, c.scheduler AS scheduler,
           c.request_retention AS request_retention, c.daily_new_limit AS daily_new_limit,
           c.daily_review_limit AS daily_review_limit
    FROM stack_identity i LEFT JOIN stack_config c ON c.folder = i.folder;

ALTER TABLE sm2_state ADD COLUMN last_review_ms INTEGER NOT NULL DEFAULT 0;
";

/// The run plan is stored with the run. The item filter is not: the resulting queue is
/// (`run_queue`). Defaults match the spaced plan so runs created before this step stay valid.
const V2: &str = "
ALTER TABLE run ADD COLUMN order_name TEXT NOT NULL DEFAULT 'due'
    CHECK (order_name IN ('due', 'sequential', 'shuffled', 'hardest_first', 'easiest_first',
                          'stalest_first', 'lowest_retrievability'));
ALTER TABLE run ADD COLUMN effect TEXT NOT NULL DEFAULT 'update_schedule'
    CHECK (effect IN ('update_schedule', 'practice_only'));
ALTER TABLE run ADD COLUMN seed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE run ADD COLUMN run_limit INTEGER;
ALTER TABLE run ADD COLUMN requeue_again INTEGER NOT NULL DEFAULT 1 CHECK (requeue_again IN (0, 1));
";

/// Derived index (rebuildable from the vault) followed by authoritative history. History tables
/// have no foreign keys to `item`, so deleting or renaming a file never deletes history.
const V1: &str = "
CREATE TABLE file (
    path     TEXT PRIMARY KEY,
    size     INTEGER NOT NULL,
    mtime_ns INTEGER NOT NULL,
    hash     INTEGER NOT NULL,
    uid      TEXT,
    error    TEXT
) WITHOUT ROWID;

CREATE TABLE item (
    uid        TEXT PRIMARY KEY,
    path       TEXT NOT NULL UNIQUE REFERENCES file(path) ON DELETE CASCADE ON UPDATE CASCADE,
    kind       TEXT NOT NULL CHECK (kind IN ('stack', 'note', 'card', 'question')),
    title      TEXT NOT NULL,
    stack_path TEXT,
    created_ms INTEGER,
    updated_ms INTEGER
) WITHOUT ROWID;
CREATE INDEX item_stack ON item(stack_path);
CREATE INDEX item_title ON item(title COLLATE NOCASE);

CREATE TABLE link (
    src_uid TEXT NOT NULL REFERENCES item(uid) ON DELETE CASCADE,
    seq     INTEGER NOT NULL,
    target  TEXT NOT NULL,
    heading TEXT,
    label   TEXT,
    line    INTEGER NOT NULL,
    PRIMARY KEY (src_uid, seq)
) WITHOUT ROWID;
CREATE INDEX link_target ON link(target COLLATE NOCASE);

CREATE TABLE tag (
    uid TEXT NOT NULL REFERENCES item(uid) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    PRIMARY KEY (uid, tag)
) WITHOUT ROWID;
CREATE INDEX tag_name ON tag(tag COLLATE NOCASE);

CREATE TABLE review (
    id           INTEGER PRIMARY KEY,
    item_uid     TEXT NOT NULL,
    rating       INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 4),
    at_ms        INTEGER NOT NULL,
    elapsed_days REAL NOT NULL
);
CREATE INDEX review_item ON review(item_uid, at_ms);

CREATE TABLE mem_state (
    item_uid       TEXT PRIMARY KEY,
    stability      REAL NOT NULL,
    difficulty     REAL NOT NULL,
    due_ms         INTEGER NOT NULL,
    last_review_ms INTEGER NOT NULL
) WITHOUT ROWID;
CREATE INDEX mem_state_due ON mem_state(due_ms);

CREATE TABLE sm2_state (
    item_uid      TEXT PRIMARY KEY,
    ease          REAL NOT NULL,
    repetitions   INTEGER NOT NULL,
    interval_days INTEGER NOT NULL,
    due_ms        INTEGER NOT NULL
) WITHOUT ROWID;

CREATE TABLE run (
    uid           TEXT PRIMARY KEY,
    mode          TEXT NOT NULL CHECK (mode IN ('flip', 'answer')),
    state         TEXT NOT NULL CHECK (state IN ('idle', 'running', 'paused', 'finished', 'aborted')),
    started_at_ms INTEGER
) WITHOUT ROWID;

CREATE TABLE run_queue (
    run_uid  TEXT NOT NULL,
    pos      INTEGER NOT NULL,
    item_uid TEXT NOT NULL,
    PRIMARY KEY (run_uid, pos)
) WITHOUT ROWID;

CREATE TABLE run_event (
    run_uid  TEXT NOT NULL,
    seq      INTEGER NOT NULL,
    t_ms     INTEGER NOT NULL,
    item_uid TEXT,
    kind     TEXT NOT NULL,
    payload  TEXT,
    PRIMARY KEY (run_uid, seq)
) WITHOUT ROWID;
";
