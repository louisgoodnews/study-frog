//! Schema step 28: timetable entries for lectures and lessons (STORY-0141).

pub(crate) const V28: &str = "
CREATE TABLE timetable_entry (
    id           TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    day_of_week  INTEGER NOT NULL,
    start_minute INTEGER NOT NULL,
    duration_min INTEGER NOT NULL,
    location     TEXT,
    instructor   TEXT,
    stack_uid    TEXT
);
CREATE INDEX idx_timetable_day ON timetable_entry(day_of_week, start_minute);
";
