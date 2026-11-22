//! Custom learn-mode runs in the store (STORY-0198): which runs are custom, the graph each ran, and the path each
//! item took through it. Built-in readers leave these runs out with [`BUILTIN_RUNS`].

use rusqlite::{Connection, OptionalExtension, params};

use crate::error::{StoreError, sql_err};

/// One step of an item's path: the node, its kind, and the rating given there.
pub type PathStep = (String, String, Option<String>);

/// One entered node of a custom run, as stored.
pub struct StepRow<'a> {
    /// The run.
    pub run_uid: &'a str,
    /// The step's place in the run.
    pub seq: i64,
    /// The item the node was entered for.
    pub item_uid: &'a str,
    /// The node entered.
    pub node_id: &'a str,
    /// What kind of node it is.
    pub node_kind: &'a str,
    /// The branch condition taken to get there.
    pub edge_when: Option<&'a str>,
    /// When, in milliseconds.
    pub t_ms: i64,
}

/// A SQL condition for `run.uid` that keeps only the built-in runs (those with no custom-mode record).
pub const BUILTIN_RUNS: &str = "uid NOT IN (SELECT run_uid FROM learn_run)";

/// Mark run `run_uid` as a run of the custom mode `mode_id`, keeping the graph it runs (`graph_toml`).
pub fn insert_learn_run(
    conn: &Connection,
    run_uid: &str,
    mode_id: &str,
    mode_version: i64,
    graph_hash: &str,
    graph_toml: &str,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO learn_run (run_uid, mode_id, mode_version, graph_hash, graph_toml) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![run_uid, mode_id, mode_version, graph_hash, graph_toml],
    )
    .map_err(sql_err)?;
    Ok(())
}

/// The mode and the graph text of a custom run; `None` for a built-in run.
pub fn learn_run_of(
    conn: &Connection,
    run_uid: &str,
) -> Result<Option<(String, String)>, StoreError> {
    conn.query_row(
        "SELECT mode_id, graph_toml FROM learn_run WHERE run_uid = ?1",
        params![run_uid],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
    .map_err(sql_err)
}

/// The next sequence number of a custom run's steps (0 for the first).
pub fn next_learn_seq(conn: &Connection, run_uid: &str) -> Result<i64, StoreError> {
    conn.query_row(
        "SELECT COALESCE(MAX(seq) + 1, 0) FROM learn_step WHERE run_uid = ?1",
        params![run_uid],
        |r| r.get(0),
    )
    .map_err(sql_err)
}

/// The nodes an item has entered in a custom run, in order: (node id, node kind, rating given there).
pub fn item_path(
    conn: &Connection,
    run_uid: &str,
    item_uid: &str,
) -> Result<Vec<PathStep>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT node_id, node_kind, rating FROM learn_step WHERE run_uid = ?1 AND item_uid = ?2 ORDER BY seq")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map(params![run_uid, item_uid], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(sql_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_err)
}

/// Append one entered node to a custom run's steps.
pub fn insert_learn_step(conn: &Connection, row: &StepRow) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO learn_step (run_uid, seq, item_uid, node_id, node_kind, edge_when, t_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![row.run_uid, row.seq, row.item_uid, row.node_id, row.node_kind, row.edge_when, row.t_ms],
    )
    .map_err(sql_err)?;
    Ok(())
}

/// Record the rating an item got at its rate node: the last step of the item holds it.
pub fn set_last_rating(
    conn: &Connection,
    run_uid: &str,
    item_uid: &str,
    rating: &str,
) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE learn_step SET rating = ?3 WHERE run_uid = ?1 AND item_uid = ?2 \
         AND seq = (SELECT MAX(seq) FROM learn_step WHERE run_uid = ?1 AND item_uid = ?2)",
        params![run_uid, item_uid, rating],
    )
    .map_err(sql_err)?;
    Ok(())
}

/// Per custom mode: (mode id, runs, steps), for the custom report (opt-in; never shown with the built-in ones).
pub fn custom_mode_totals(conn: &Connection) -> Result<Vec<(String, i64, i64)>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT l.mode_id, COUNT(DISTINCT l.run_uid), \
             (SELECT COUNT(*) FROM learn_step s JOIN learn_run r2 ON r2.run_uid = s.run_uid WHERE r2.mode_id = l.mode_id) \
             FROM learn_run l GROUP BY l.mode_id ORDER BY l.mode_id",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(sql_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_err)
}
