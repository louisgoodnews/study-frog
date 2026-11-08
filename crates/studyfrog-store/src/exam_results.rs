//! Graded exams, one row per exam run (`docs/specs/exam-mode.md`).

use rusqlite::{Connection, OptionalExtension, Row, params};
use studyfrog_core::exam::{ExamResult, grade_from_letter, grade_letter};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;

use crate::error::{StoreError, sql_err};
use crate::runs::{corrupt, uid_of};

/// Store (or replace) the result of an exam run.
pub fn save_exam_result(conn: &Connection, r: &ExamResult) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR REPLACE INTO exam_result (run_uid, items, answered, percent, grade, passed, engine, graded_at_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            format_ulid(r.run),
            r.items,
            r.answered,
            f64::from(r.percent),
            grade_letter(r.grade),
            r.passed,
            r.engine,
            r.graded_at_ms
        ],
    )
    .map(drop)
    .map_err(sql_err)
}

type Raw = (String, u32, u32, f64, String, bool, String, i64);

fn raw(r: &Row<'_>) -> rusqlite::Result<Raw> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get(6)?,
        r.get(7)?,
    ))
}

fn result_of(
    (run, items, answered, percent, grade, passed, engine, at): Raw,
) -> Result<ExamResult, StoreError> {
    Ok(ExamResult {
        run: uid_of(&run)?,
        items,
        answered,
        percent: percent as f32,
        grade: grade_from_letter(&grade).ok_or_else(|| corrupt("grade", &grade))?,
        passed,
        engine,
        graded_at_ms: at,
    })
}

const COLUMNS: &str = "run_uid, items, answered, percent, grade, passed, engine, graded_at_ms";

/// The result of an exam run, if it was graded.
pub fn load_exam_result(conn: &Connection, run: Uid) -> Result<Option<ExamResult>, StoreError> {
    let sql = format!("SELECT {COLUMNS} FROM exam_result WHERE run_uid = ?1");
    let found = conn
        .query_row(&sql, [format_ulid(run)], raw)
        .optional()
        .map_err(sql_err)?;
    found.map(result_of).transpose()
}

/// The latest graded exams, newest first.
pub fn recent_exam_results(conn: &Connection, limit: usize) -> Result<Vec<ExamResult>, StoreError> {
    let sql = format!("SELECT {COLUMNS} FROM exam_result ORDER BY graded_at_ms DESC LIMIT ?1");
    let mut stmt = conn.prepare(&sql).map_err(sql_err)?;
    let rows = stmt.query_map([limit as i64], raw).map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(result_of(row.map_err(sql_err)?)?);
    }
    Ok(out)
}
