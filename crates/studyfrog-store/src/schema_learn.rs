//! Schema step 31: custom learn-mode runs (STORY-0198, `docs/research/learn-mode-proposal.md`). A custom run
//! is an ordinary `run` row (mode `flip`, effect `practice_only`); `learn_run` marks it as custom and keeps the
//! graph it ran (a full snapshot), and `learn_step` keeps the path each item took through the graph.

/// The two tables. Only additive `CREATE TABLE`s: no existing table changes, and built-in readers filter the
/// custom runs out by `learn_run` membership.
pub(crate) const V31: &str = "
CREATE TABLE learn_run (
    run_uid         TEXT PRIMARY KEY,
    mode_id         TEXT NOT NULL,
    mode_version    INTEGER NOT NULL CHECK (mode_version >= 1),
    graph_hash      TEXT NOT NULL,
    graph_toml      TEXT NOT NULL,
    schedule_effect TEXT NOT NULL DEFAULT 'practice_only' CHECK (schedule_effect IN ('practice_only'))
) WITHOUT ROWID;
CREATE TABLE learn_step (
    run_uid   TEXT NOT NULL,
    seq       INTEGER NOT NULL CHECK (seq >= 0),
    item_uid  TEXT NOT NULL,
    node_id   TEXT NOT NULL,
    node_kind TEXT NOT NULL,
    edge_when TEXT,
    rating    TEXT CHECK (rating IS NULL OR rating IN ('again', 'hard', 'good', 'easy')),
    t_ms      INTEGER NOT NULL,
    PRIMARY KEY (run_uid, seq)
) WITHOUT ROWID;
CREATE INDEX learn_step_item ON learn_step (run_uid, item_uid);
";
