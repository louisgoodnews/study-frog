//! Schema step 23: diary entries and bidirectional links (TASK-0171, STORY-0068).

/// Diary entries indexed from vault Markdown files along with links to runs, stacks, items, and exams.
pub(crate) const V23: &str = "
CREATE TABLE diary_entries (
    uid           TEXT PRIMARY KEY,
    path          TEXT NOT NULL UNIQUE REFERENCES file(path) ON DELETE CASCADE ON UPDATE CASCADE,
    date          TEXT NOT NULL,
    title         TEXT NOT NULL,
    study_minutes INTEGER,
    ratings_json  TEXT NOT NULL,
    content       TEXT NOT NULL,
    mtime         INTEGER NOT NULL
) WITHOUT ROWID;

CREATE INDEX diary_entries_date ON diary_entries(date);

CREATE TABLE diary_runs (
    diary_uid TEXT NOT NULL REFERENCES diary_entries(uid) ON DELETE CASCADE,
    run_uid   TEXT NOT NULL,
    PRIMARY KEY (diary_uid, run_uid)
) WITHOUT ROWID;
CREATE INDEX diary_runs_run ON diary_runs(run_uid);

CREATE TABLE diary_stacks (
    diary_uid TEXT NOT NULL REFERENCES diary_entries(uid) ON DELETE CASCADE,
    stack_ref TEXT NOT NULL,
    PRIMARY KEY (diary_uid, stack_ref)
) WITHOUT ROWID;
CREATE INDEX diary_stacks_ref ON diary_stacks(stack_ref);

CREATE TABLE diary_items (
    diary_uid TEXT NOT NULL REFERENCES diary_entries(uid) ON DELETE CASCADE,
    item_uid  TEXT NOT NULL,
    PRIMARY KEY (diary_uid, item_uid)
) WITHOUT ROWID;
CREATE INDEX diary_items_item ON diary_items(item_uid);

CREATE TABLE diary_exams (
    diary_uid TEXT NOT NULL REFERENCES diary_entries(uid) ON DELETE CASCADE,
    exam_id   TEXT NOT NULL,
    PRIMARY KEY (diary_uid, exam_id)
) WITHOUT ROWID;
CREATE INDEX diary_exams_exam ON diary_exams(exam_id);

CREATE TABLE diary_tags (
    diary_uid TEXT NOT NULL REFERENCES diary_entries(uid) ON DELETE CASCADE,
    tag       TEXT NOT NULL,
    PRIMARY KEY (diary_uid, tag)
) WITHOUT ROWID;
CREATE INDEX diary_tags_name ON diary_tags(tag COLLATE NOCASE);
";
