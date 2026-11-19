//! Schema migration V29: homework items, checklists, reminders, and fired-reminder log.

/// Migration 29 SQL creating homework persistence tables and indexes.
pub const V29: &str = "
CREATE TABLE homework (
    id                TEXT PRIMARY KEY,
    title             TEXT NOT NULL,
    subject           TEXT,
    stack_uid         TEXT,
    timetable_id      TEXT,
    due_date          TEXT NOT NULL,
    due_minute        INTEGER NOT NULL CHECK (due_minute BETWEEN 0 AND 1439),
    start_date        TEXT,
    estimated_minutes INTEGER NOT NULL CHECK (estimated_minutes BETWEEN 1 AND 525600),
    priority          TEXT NOT NULL CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    status            TEXT NOT NULL CHECK (status IN ('not_started', 'in_progress', 'completed')),
    notes             TEXT NOT NULL DEFAULT ''
) WITHOUT ROWID;
CREATE INDEX idx_homework_due ON homework(due_date, due_minute, id);
CREATE INDEX idx_homework_status ON homework(status, due_date);

CREATE TABLE homework_checklist (
    homework_id TEXT NOT NULL REFERENCES homework(id) ON DELETE CASCADE,
    pos         INTEGER NOT NULL CHECK (pos >= 0),
    text        TEXT NOT NULL,
    done        INTEGER NOT NULL DEFAULT 0 CHECK (done IN (0, 1)),
    PRIMARY KEY (homework_id, pos)
) WITHOUT ROWID;

CREATE TABLE homework_reminder (
    homework_id    TEXT NOT NULL REFERENCES homework(id) ON DELETE CASCADE,
    minutes_before INTEGER NOT NULL CHECK (minutes_before >= 0),
    PRIMARY KEY (homework_id, minutes_before)
) WITHOUT ROWID;

CREATE TABLE homework_reminder_fired (
    homework_id    TEXT NOT NULL REFERENCES homework(id) ON DELETE CASCADE,
    due_date       TEXT NOT NULL,
    minutes_before INTEGER NOT NULL CHECK (minutes_before >= 0),
    PRIMARY KEY (homework_id, due_date, minutes_before)
) WITHOUT ROWID;
";
