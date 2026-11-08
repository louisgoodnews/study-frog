//! Schema step 26: focus blocks of the focus timer (STORY-0112, `docs/specs/focus-timer.md`). History, not
//! derived: one row per finished work block, with the calendar session that was on when it ended.
pub(crate) const V26: &str = "
CREATE TABLE focus_block (
    id           INTEGER PRIMARY KEY,
    started_ms   INTEGER NOT NULL,
    ended_ms     INTEGER NOT NULL CHECK (ended_ms >= started_ms),
    minutes      INTEGER NOT NULL CHECK (minutes BETWEEN 0 AND 1440),
    stack        TEXT,
    session_uid  TEXT,
    session_date TEXT
);
CREATE INDEX focus_block_session ON focus_block(session_uid, session_date);
CREATE INDEX focus_block_time ON focus_block(ended_ms);
";
