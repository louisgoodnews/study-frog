//! The second and third schema versions: run plans, per-stack settings and the run filter.

/// The run plan is stored with the run. The item filter is not: the resulting queue is
/// (`run_queue`). Defaults match the spaced plan so runs created before this step stay valid.
pub(crate) const V2: &str = "
ALTER TABLE run ADD COLUMN order_name TEXT NOT NULL DEFAULT 'due'
    CHECK (order_name IN ('due', 'sequential', 'shuffled', 'hardest_first', 'easiest_first',
                          'stalest_first', 'lowest_retrievability'));
ALTER TABLE run ADD COLUMN effect TEXT NOT NULL DEFAULT 'update_schedule'
    CHECK (effect IN ('update_schedule', 'practice_only'));
ALTER TABLE run ADD COLUMN seed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE run ADD COLUMN run_limit INTEGER;
ALTER TABLE run ADD COLUMN requeue_again INTEGER NOT NULL DEFAULT 1 CHECK (requeue_again IN (0, 1));
";

/// The run filter (list-valued parts in their own tables, with a position to keep their order),
/// per-stack identity and settings (derived index, each following its own file), and the
/// last-review time SM-2 items need for queue building.
pub(crate) const V3: &str = "
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
