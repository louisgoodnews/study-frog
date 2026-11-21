//! Schema step 29: homework items and their checklist steps (STORY-0165).

pub(crate) const V29: &str = "
CREATE TABLE homework_item (
    id           TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    course       TEXT,
    start_date   TEXT,
    due_date     TEXT NOT NULL,
    due_minute   INTEGER NOT NULL,
    estimate_min INTEGER NOT NULL,
    priority     TEXT NOT NULL,
    status       TEXT NOT NULL,
    notes        TEXT NOT NULL DEFAULT ''
);
CREATE INDEX idx_homework_due ON homework_item(due_date, due_minute);
CREATE TABLE homework_step (
    homework_id TEXT NOT NULL,
    position    INTEGER NOT NULL,
    text        TEXT NOT NULL,
    done        INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (homework_id, position)
);
CREATE TABLE homework_fired (
    homework_id TEXT NOT NULL,
    lead_min    INTEGER NOT NULL,
    PRIMARY KEY (homework_id, lead_min)
);
";
