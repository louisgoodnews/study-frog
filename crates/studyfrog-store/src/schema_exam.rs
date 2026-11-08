//! Schema step 25: exam runs and their results (EPIC-033, `docs/specs/exam-mode.md`).

/// The `run` table is rebuilt (a CHECK cannot change in place; no table refers to it) so its mode may be
/// `exam`; every row is kept. `exam_result` holds one graded result per exam run; it is history, not derived.
pub(crate) const V25: &str = "
CREATE TABLE run_new (
    uid             TEXT PRIMARY KEY,
    mode            TEXT NOT NULL CHECK (mode IN ('flip', 'answer', 'exam')),
    state           TEXT NOT NULL CHECK (state IN ('idle', 'running', 'paused', 'finished', 'aborted')),
    started_at_ms   INTEGER,
    order_name      TEXT NOT NULL DEFAULT 'due'
        CHECK (order_name IN ('due', 'sequential', 'shuffled', 'hardest_first', 'easiest_first',
                              'stalest_first', 'lowest_retrievability', 'least_familiar')),
    effect          TEXT NOT NULL DEFAULT 'update_schedule' CHECK (effect IN ('update_schedule', 'practice_only')),
    seed            INTEGER NOT NULL DEFAULT 0,
    run_limit       INTEGER,
    requeue_again   INTEGER NOT NULL DEFAULT 1 CHECK (requeue_again IN (0, 1)),
    filter_state    TEXT NOT NULL DEFAULT 'any' CHECK (filter_state IN ('any', 'new', 'learned', 'due', 'not_due')),
    filter_diff_min REAL,
    filter_diff_max REAL,
    pacing          TEXT NOT NULL DEFAULT 'free' CHECK (pacing IN ('free', 'countdown', 'count_up')),
    countdown_secs  INTEGER CHECK (countdown_secs IS NULL OR countdown_secs BETWEEN 60 AND 10800),
    shuffle         INTEGER NOT NULL DEFAULT 0 CHECK (shuffle IN (0, 1)),
    filter_fam_below REAL CHECK (filter_fam_below IS NULL OR filter_fam_below BETWEEN 0 AND 1)
) WITHOUT ROWID;
INSERT INTO run_new SELECT uid, mode, state, started_at_ms, order_name, effect, seed, run_limit, requeue_again,
       filter_state, filter_diff_min, filter_diff_max, pacing, countdown_secs, shuffle, filter_fam_below FROM run;
DROP TABLE run;
ALTER TABLE run_new RENAME TO run;
CREATE TABLE exam_result (
    run_uid      TEXT PRIMARY KEY,
    items        INTEGER NOT NULL CHECK (items >= 0),
    answered     INTEGER NOT NULL CHECK (answered >= 0),
    percent      REAL NOT NULL CHECK (percent BETWEEN 0 AND 100),
    grade        TEXT NOT NULL CHECK (grade IN ('A+', 'A', 'A-', 'B+', 'B', 'B-', 'C', 'D', 'F')),
    passed       INTEGER NOT NULL CHECK (passed IN (0, 1)),
    engine       TEXT NOT NULL DEFAULT '',
    graded_at_ms INTEGER NOT NULL
) WITHOUT ROWID;
";
