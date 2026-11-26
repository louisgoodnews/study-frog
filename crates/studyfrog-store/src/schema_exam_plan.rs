//! SQLite migration V35: exam scheduler plans, targets, and daily schedule blocks (EPIC-076, TASK-0306).

/// Schema migration V35.
pub const V35: &str = "
CREATE TABLE exam_plan (
    id           TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    exam_date    TEXT NOT NULL,
    window_mode  TEXT NOT NULL CHECK (window_mode IN ('user', 'proposed')),
    window_weeks INTEGER NOT NULL,
    reasons_json TEXT NOT NULL,
    created_ms   INTEGER NOT NULL,
    updated_ms   INTEGER NOT NULL
) WITHOUT ROWID;

CREATE TABLE exam_target (
    plan_id      TEXT NOT NULL,
    pos          INTEGER NOT NULL,
    kind         TEXT NOT NULL CHECK (kind IN ('stack', 'item')),
    target_value TEXT NOT NULL,
    PRIMARY KEY (plan_id, pos),
    FOREIGN KEY (plan_id) REFERENCES exam_plan(id) ON DELETE CASCADE
) WITHOUT ROWID;

CREATE TABLE exam_schedule_block (
    plan_id        TEXT NOT NULL,
    block_date     TEXT NOT NULL,
    item_uids_json TEXT NOT NULL,
    new_count      INTEGER NOT NULL,
    review_count   INTEGER NOT NULL,
    PRIMARY KEY (plan_id, block_date),
    FOREIGN KEY (plan_id) REFERENCES exam_plan(id) ON DELETE CASCADE
) WITHOUT ROWID;
CREATE INDEX exam_schedule_block_date ON exam_schedule_block(block_date);
";
