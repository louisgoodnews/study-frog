//! SQL of the later migrations (v4 and up), kept apart so `schema.rs` stays short.

/// How typed answers compared with the reference (history, no foreign keys). `engine` names the
/// runtime that compared them, because scores of different runtimes are not interchangeable.
/// Shuffling is a switch of its own: it can go with any order. Runs stored with the old order
/// `shuffled` become sequential runs with the switch on.
pub(crate) const V7: &str = "
ALTER TABLE run ADD COLUMN shuffle INTEGER NOT NULL DEFAULT 0 CHECK (shuffle IN (0, 1));
UPDATE run SET shuffle = 1, order_name = 'sequential' WHERE order_name = 'shuffled';
";

pub(crate) const V6: &str = "
CREATE TABLE answer_score (
    id       INTEGER PRIMARY KEY,
    run_uid  TEXT NOT NULL,
    item_uid TEXT NOT NULL,
    value    REAL NOT NULL CHECK (value BETWEEN 0 AND 1),
    verdict  TEXT NOT NULL CHECK (verdict IN ('correct', 'partial', 'incorrect')),
    engine   TEXT NOT NULL,
    at_ms    INTEGER NOT NULL
);
CREATE INDEX answer_score_run ON answer_score(run_uid);
";

/// Achievement unlocks (history, no foreign keys); `scope_key` is empty when global, else a stack uid.
pub(crate) const V5: &str = "
CREATE TABLE achievement_unlock (
    rule_id     TEXT NOT NULL,
    scope_key   TEXT NOT NULL,
    unlocked_ms INTEGER NOT NULL,
    PRIMARY KEY (rule_id, scope_key)
) WITHOUT ROWID;
";

/// Authoritative planning data (ADR 0003): the calendar's sessions and the kanban board. Dates are
/// ISO `YYYY-MM-DD` text. Cards belong to a column and are ordered by `pos`; deleting a column
/// would delete its cards, so the service moves them away first.
pub(crate) const V4: &str = "
CREATE TABLE session (
    id            TEXT PRIMARY KEY,
    title         TEXT NOT NULL,
    stack_uid     TEXT,
    start_date    TEXT NOT NULL,
    start_minute  INTEGER NOT NULL CHECK (start_minute BETWEEN 0 AND 1439),
    duration_min  INTEGER NOT NULL CHECK (duration_min BETWEEN 1 AND 1440),
    repeat_kind   TEXT NOT NULL CHECK (repeat_kind IN ('never', 'daily', 'weekly', 'monthly')),
    repeat_every  INTEGER NOT NULL DEFAULT 1 CHECK (repeat_every >= 1),
    repeat_days   INTEGER NOT NULL DEFAULT 0 CHECK (repeat_days BETWEEN 0 AND 127),
    ends_kind     TEXT NOT NULL CHECK (ends_kind IN ('forever', 'on', 'after')),
    ends_date     TEXT,
    ends_count    INTEGER,
    notes         TEXT NOT NULL DEFAULT ''
) WITHOUT ROWID;
CREATE INDEX session_start ON session(start_date);

CREATE TABLE session_cancelled (
    session_id TEXT NOT NULL REFERENCES session(id) ON DELETE CASCADE,
    date       TEXT NOT NULL,
    PRIMARY KEY (session_id, date)
) WITHOUT ROWID;

CREATE TABLE board_column (
    id        TEXT PRIMARY KEY,
    pos       INTEGER NOT NULL,
    name      TEXT NOT NULL,
    wip_limit INTEGER CHECK (wip_limit IS NULL OR wip_limit >= 0)
) WITHOUT ROWID;

CREATE TABLE board_card (
    id        TEXT PRIMARY KEY,
    column_id TEXT NOT NULL REFERENCES board_column(id) ON DELETE CASCADE,
    pos       INTEGER NOT NULL,
    title     TEXT NOT NULL,
    notes     TEXT NOT NULL DEFAULT '',
    stack_uid TEXT,
    points    INTEGER
) WITHOUT ROWID;
CREATE INDEX board_card_column ON board_card(column_id, pos);
";
