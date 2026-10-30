//! How typed answers compared with their reference, stored with the run they belong to.

use rusqlite::{Connection, params};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;

use crate::error::{StoreError, sql_err};
use crate::runs::{corrupt, uid_of};

/// One stored comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct AnswerScoreRow {
    /// The run.
    pub run: Uid,
    /// The item answered.
    pub item: Uid,
    /// 0 to 1.
    pub value: f32,
    /// `correct`, `partial` or `incorrect`.
    pub verdict: String,
    /// The runtime that compared the texts.
    pub engine: String,
    /// When, in Unix milliseconds.
    pub at_ms: i64,
}

/// Append a comparison.
pub fn insert_answer_score(conn: &Connection, r: &AnswerScoreRow) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO answer_score (run_uid, item_uid, value, verdict, engine, at_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![format_ulid(r.run), format_ulid(r.item), r.value, r.verdict, r.engine, r.at_ms],
    )
    .map(drop)
    .map_err(sql_err)
}

/// The comparisons of a run, oldest first.
pub fn load_answer_scores(conn: &Connection, run: Uid) -> Result<Vec<AnswerScoreRow>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT item_uid, value, verdict, engine, at_ms FROM answer_score WHERE run_uid = ?1 ORDER BY id")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([format_ulid(run)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, f32>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (item, value, verdict, engine, at_ms) = row.map_err(sql_err)?;
        if verdict.is_empty() {
            return Err(corrupt("verdict", &verdict));
        }
        out.push(AnswerScoreRow {
            run,
            item: uid_of(&item)?,
            value,
            verdict,
            engine,
            at_ms,
        });
    }
    Ok(out)
}
