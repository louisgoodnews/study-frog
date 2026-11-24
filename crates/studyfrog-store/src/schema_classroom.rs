//! Schema steps 32 and 33: classroom sessions (EPIC-068, STORY-0208). Normalized tables for the session header,
//! its participants, ordered chat turns, assignments, group rosters, submissions and grades. Every child
//! table cascades from `classroom_session`, so deleting a session removes all of it in one statement.

/// Additive only: new tables, no existing table changes.
pub(crate) const V32: &str = "
CREATE TABLE classroom_session (
    uid           TEXT PRIMARY KEY,
    title         TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL
) WITHOUT ROWID;
CREATE INDEX classroom_session_created ON classroom_session (created_at_ms);

CREATE TABLE classroom_participant (
    session_uid TEXT NOT NULL REFERENCES classroom_session (uid) ON DELETE CASCADE,
    idx         INTEGER NOT NULL CHECK (idx >= 0),
    kind        TEXT NOT NULL CHECK (kind IN ('teacher', 'simulated_student', 'user')),
    name        TEXT NOT NULL,
    PRIMARY KEY (session_uid, idx)
) WITHOUT ROWID;

CREATE TABLE classroom_turn (
    session_uid TEXT NOT NULL REFERENCES classroom_session (uid) ON DELETE CASCADE,
    seq         INTEGER NOT NULL CHECK (seq >= 0),
    speaker     INTEGER NOT NULL,
    at_ms       INTEGER NOT NULL,
    text        TEXT NOT NULL,
    PRIMARY KEY (session_uid, seq),
    FOREIGN KEY (session_uid, speaker) REFERENCES classroom_participant (session_uid, idx)
) WITHOUT ROWID;

CREATE TABLE classroom_assignment (
    session_uid TEXT NOT NULL REFERENCES classroom_session (uid) ON DELETE CASCADE,
    uid         TEXT NOT NULL,
    seq         INTEGER NOT NULL CHECK (seq >= 0),
    title       TEXT NOT NULL,
    prompt      TEXT NOT NULL,
    kind        TEXT NOT NULL CHECK (kind IN ('solo', 'group')),
    assignee    INTEGER,
    PRIMARY KEY (session_uid, uid),
    CHECK ((kind = 'solo') = (assignee IS NOT NULL)),
    FOREIGN KEY (session_uid, assignee) REFERENCES classroom_participant (session_uid, idx)
) WITHOUT ROWID;

CREATE TABLE classroom_group (
    session_uid    TEXT NOT NULL,
    assignment_uid TEXT NOT NULL,
    idx            INTEGER NOT NULL CHECK (idx >= 0),
    name           TEXT NOT NULL,
    PRIMARY KEY (session_uid, assignment_uid, idx),
    FOREIGN KEY (session_uid, assignment_uid) REFERENCES classroom_assignment (session_uid, uid) ON DELETE CASCADE
) WITHOUT ROWID;

CREATE TABLE classroom_group_member (
    session_uid    TEXT NOT NULL,
    assignment_uid TEXT NOT NULL,
    group_idx      INTEGER NOT NULL,
    seq            INTEGER NOT NULL CHECK (seq >= 0),
    participant    INTEGER NOT NULL,
    PRIMARY KEY (session_uid, assignment_uid, group_idx, seq),
    UNIQUE (session_uid, assignment_uid, participant),
    FOREIGN KEY (session_uid, assignment_uid, group_idx) REFERENCES classroom_group (session_uid, assignment_uid, idx) ON DELETE CASCADE,
    FOREIGN KEY (session_uid, participant) REFERENCES classroom_participant (session_uid, idx)
) WITHOUT ROWID;

CREATE TABLE classroom_submission (
    session_uid     TEXT NOT NULL,
    seq             INTEGER NOT NULL CHECK (seq >= 0),
    assignment_uid  TEXT NOT NULL,
    owner_kind      TEXT NOT NULL CHECK (owner_kind IN ('participant', 'group')),
    owner_idx       INTEGER NOT NULL CHECK (owner_idx >= 0),
    text            TEXT NOT NULL,
    submitted_at_ms INTEGER NOT NULL,
    PRIMARY KEY (session_uid, seq),
    UNIQUE (session_uid, assignment_uid, owner_kind, owner_idx),
    FOREIGN KEY (session_uid, assignment_uid) REFERENCES classroom_assignment (session_uid, uid) ON DELETE CASCADE
) WITHOUT ROWID;

CREATE TABLE classroom_grade (
    session_uid    TEXT NOT NULL,
    assignment_uid TEXT NOT NULL,
    owner_kind     TEXT NOT NULL,
    owner_idx      INTEGER NOT NULL,
    points         INTEGER NOT NULL CHECK (points >= 0),
    max            INTEGER NOT NULL CHECK (max > 0 AND points <= max),
    feedback       TEXT NOT NULL,
    graded_at_ms   INTEGER NOT NULL,
    PRIMARY KEY (session_uid, assignment_uid, owner_kind, owner_idx),
    FOREIGN KEY (session_uid, assignment_uid, owner_kind, owner_idx)
        REFERENCES classroom_submission (session_uid, assignment_uid, owner_kind, owner_idx) ON DELETE CASCADE
) WITHOUT ROWID;

CREATE TABLE classroom_source (
    session_uid TEXT NOT NULL REFERENCES classroom_session (uid) ON DELETE CASCADE,
    idx         INTEGER NOT NULL CHECK (idx >= 0),
    kind        TEXT NOT NULL CHECK (kind IN ('stack', 'note', 'pdf')),
    item_uid    TEXT NOT NULL,
    PRIMARY KEY (session_uid, idx),
    UNIQUE (session_uid, kind, item_uid)
) WITHOUT ROWID;
";

/// Additive: the grading points of each assignment, in order (migration 33).
pub(crate) const V33: &str = "
CREATE TABLE classroom_rubric (
    session_uid    TEXT NOT NULL,
    assignment_uid TEXT NOT NULL,
    idx            INTEGER NOT NULL CHECK (idx >= 0),
    text           TEXT NOT NULL,
    PRIMARY KEY (session_uid, assignment_uid, idx),
    FOREIGN KEY (session_uid, assignment_uid) REFERENCES classroom_assignment (session_uid, uid) ON DELETE CASCADE
) WITHOUT ROWID;
";
