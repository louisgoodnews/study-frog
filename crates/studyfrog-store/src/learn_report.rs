//! The custom report of one learn mode (STORY-0198, opt-in): its runs, the items it reached, the nodes entered, the
//! ratings given and the branches taken. It reads custom runs only; the built-in reports never include them.

use rusqlite::{Connection, params};

use crate::error::{StoreError, sql_err};

/// What one learn mode's runs did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModeReport {
    /// The mode's id (its file name without `.toml`).
    pub mode_id: String,
    /// Runs of the mode.
    pub runs: i64,
    /// Of those, the runs that finished.
    pub finished: i64,
    /// Distinct items the mode showed.
    pub items: i64,
    /// Entries per node id, sorted by id.
    pub visits: Vec<(String, i64)>,
    /// Ratings given, per word (`again`, `hard`, `good`, `easy`).
    pub ratings: Vec<(String, i64)>,
    /// Branch wires taken, per condition word.
    pub branches: Vec<(String, i64)>,
}

fn pairs(conn: &Connection, sql: &str, mode: &str) -> Result<Vec<(String, i64)>, StoreError> {
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map(params![mode], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(sql_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_err)
}

/// The report of the custom mode `mode_id`.
pub fn mode_report(conn: &Connection, mode_id: &str) -> Result<ModeReport, StoreError> {
    let count = |sql: &str| -> Result<i64, StoreError> {
        conn.query_row(sql, params![mode_id], |r| r.get(0))
            .map_err(sql_err)
    };
    let runs = count("SELECT COUNT(*) FROM learn_run WHERE mode_id = ?1")?;
    let finished = count(
        "SELECT COUNT(*) FROM learn_run l JOIN run r ON r.uid = l.run_uid \
         WHERE l.mode_id = ?1 AND r.state = 'finished'",
    )?;
    let items = count(
        "SELECT COUNT(DISTINCT s.item_uid) FROM learn_step s JOIN learn_run l ON l.run_uid = s.run_uid \
         WHERE l.mode_id = ?1",
    )?;
    let visits = pairs(
        conn,
        "SELECT s.node_id, COUNT(*) FROM learn_step s JOIN learn_run l ON l.run_uid = s.run_uid \
         WHERE l.mode_id = ?1 GROUP BY s.node_id ORDER BY s.node_id",
        mode_id,
    )?;
    let ratings = pairs(
        conn,
        "SELECT s.rating, COUNT(*) FROM learn_step s JOIN learn_run l ON l.run_uid = s.run_uid \
         WHERE l.mode_id = ?1 AND s.rating IS NOT NULL GROUP BY s.rating ORDER BY s.rating",
        mode_id,
    )?;
    let branches = pairs(
        conn,
        "SELECT s.edge_when, COUNT(*) FROM learn_step s JOIN learn_run l ON l.run_uid = s.run_uid \
         WHERE l.mode_id = ?1 AND s.edge_when IS NOT NULL GROUP BY s.edge_when ORDER BY s.edge_when",
        mode_id,
    )?;
    Ok(ModeReport {
        mode_id: mode_id.to_owned(),
        runs,
        finished,
        items,
        visits,
        ratings,
        branches,
    })
}
