//! Schema as data. See `docs/specs/storage-schema.md`.

use crate::migrate::Migration;
use crate::schema_diary::V23;
use crate::schema_early::{V2, V3};
use crate::schema_keys::V24;
use crate::schema_later::{
    V4, V5, V6, V7, V8, V9, V10, V11, V12, V13, V14, V15, V16, V17, V18, V19,
};
use crate::schema_memory::{V21, V22};
use crate::schema_refs::V20;

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
    Migration {
        version: 13,
        name: "item views",
        sql: V13,
    },
    Migration {
        version: 14,
        name: "stack familiarity switch",
        sql: V14,
    },
    Migration {
        version: 15,
        name: "notifications",
        sql: V15,
    },
    Migration {
        version: 16,
        name: "run pacing",
        sql: V16,
    },
    Migration {
        version: 17,
        name: "answer and view timing",
        sql: V17,
    },
    Migration {
        version: 18,
        name: "item text",
        sql: V18,
    },
    Migration {
        version: 19,
        name: "least familiar order",
        sql: V19,
    },
    Migration {
        version: 20,
        name: "items referenced by several stacks",
        sql: V20,
    },
    Migration {
        version: 21,
        name: "memory per stack",
        sql: V21,
    },
    Migration {
        version: 22,
        name: "reviews per item",
        sql: V22,
    },
    Migration {
        version: 23,
        name: "diary entries and links",
        sql: V23,
    },
    Migration {
        version: 24,
        name: "item keys and colours",
        sql: V24,
    },
];

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
