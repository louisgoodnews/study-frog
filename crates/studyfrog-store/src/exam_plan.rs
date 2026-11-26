//! SQLite persistence for Exam Scheduler plans, linked targets, and daily blocks (EPIC-076, TASK-0306).

use rusqlite::{Connection, params};
use studyfrog_calendar::exam_plan::{ExamPlan, ExamScheduleBlock, ExamTarget, PrepWindow};
use studyfrog_core::civil::{format_date, parse_date};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

fn corrupt(what: &str, v: &str) -> StoreError {
    StoreError::Corrupt(format!("{what}: {v:?}"))
}

fn encode_window(w: &PrepWindow) -> (&'static str, u16, String) {
    match w {
        PrepWindow::UserWeeks(weeks) => ("user", *weeks, "[]".to_owned()),
        PrepWindow::Proposed { weeks, reasons } => (
            "proposed",
            *weeks,
            serde_json::to_string(reasons).unwrap_or_else(|_| "[]".to_owned()),
        ),
    }
}

fn decode_window(mode: &str, weeks: u16, reasons_json: &str) -> Result<PrepWindow, StoreError> {
    match mode {
        "user" => Ok(PrepWindow::UserWeeks(weeks)),
        "proposed" => {
            let reasons: Vec<String> = serde_json::from_str(reasons_json)
                .map_err(|e| corrupt("reasons_json", &e.to_string()))?;
            Ok(PrepWindow::Proposed { weeks, reasons })
        }
        other => Err(corrupt("window_mode", other)),
    }
}

/// Insert or replace an `ExamPlan`, its targets, and its schedule blocks atomically.
pub fn save_exam_plan(conn: &mut Connection, plan: &ExamPlan) -> Result<(), StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    let id_str = format_ulid(plan.id);
    let (mode, weeks, reasons_json) = encode_window(&plan.window);
    tx.execute(
        "INSERT INTO exam_plan (id, title, exam_date, window_mode, window_weeks, reasons_json, created_ms, updated_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET title = ?2, exam_date = ?3, window_mode = ?4, window_weeks = ?5,
             reasons_json = ?6, updated_ms = ?8",
        params![
            id_str,
            plan.title,
            format_date(plan.exam_date),
            mode,
            weeks,
            reasons_json,
            plan.created_ms,
            plan.updated_ms,
        ],
    )
    .map_err(sql_err)?;

    tx.execute("DELETE FROM exam_target WHERE plan_id = ?1", [&id_str])
        .map_err(sql_err)?;
    for (pos, target) in plan.targets.iter().enumerate() {
        let (kind, val) = match target {
            ExamTarget::Stack(folder) => ("stack", folder.clone()),
            ExamTarget::Item(uid) => ("item", format_ulid(*uid)),
        };
        tx.execute(
            "INSERT INTO exam_target (plan_id, pos, kind, target_value) VALUES (?1, ?2, ?3, ?4)",
            params![id_str, pos as i64, kind, val],
        )
        .map_err(sql_err)?;
    }

    tx.execute(
        "DELETE FROM exam_schedule_block WHERE plan_id = ?1",
        [&id_str],
    )
    .map_err(sql_err)?;
    for block in &plan.blocks {
        let uids: Vec<String> = block.item_uids.iter().map(|u| format_ulid(*u)).collect();
        let uids_json = serde_json::to_string(&uids).unwrap_or_else(|_| "[]".to_owned());
        tx.execute(
            "INSERT INTO exam_schedule_block (plan_id, block_date, item_uids_json, new_count, review_count)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id_str,
                format_date(block.date),
                uids_json,
                block.new_count as i64,
                block.review_count as i64,
            ],
        )
        .map_err(sql_err)?;
    }
    tx.commit().map_err(sql_err)
}

fn load_targets(conn: &Connection, id_str: &str) -> Result<Vec<ExamTarget>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT kind, target_value FROM exam_target WHERE plan_id = ?1 ORDER BY pos ASC")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([id_str], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (kind, val) = row.map_err(sql_err)?;
        match kind.as_str() {
            "stack" => out.push(ExamTarget::Stack(val)),
            "item" => {
                let uid = parse_ulid(&val).ok_or_else(|| corrupt("exam_target.item", &val))?;
                out.push(ExamTarget::Item(uid));
            }
            other => return Err(corrupt("exam_target.kind", other)),
        }
    }
    Ok(out)
}

fn load_blocks(conn: &Connection, id_str: &str) -> Result<Vec<ExamScheduleBlock>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT block_date, item_uids_json, new_count, review_count
             FROM exam_schedule_block WHERE plan_id = ?1 ORDER BY block_date ASC",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([id_str], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (date_str, uids_json, new_c, rev_c) = row.map_err(sql_err)?;
        let date = parse_date(&date_str).ok_or_else(|| corrupt("block_date", &date_str))?;
        let raw_uids: Vec<String> = serde_json::from_str(&uids_json)
            .map_err(|e| corrupt("item_uids_json", &e.to_string()))?;
        let mut item_uids = Vec::with_capacity(raw_uids.len());
        for s in raw_uids {
            item_uids.push(parse_ulid(&s).ok_or_else(|| corrupt("item_uid", &s))?);
        }
        out.push(ExamScheduleBlock {
            date,
            item_uids,
            new_count: new_c.max(0) as usize,
            review_count: rev_c.max(0) as usize,
        });
    }
    Ok(out)
}

/// Load a single `ExamPlan` by `id`.
pub fn load_exam_plan(conn: &Connection, id: Uid) -> Result<Option<ExamPlan>, StoreError> {
    let id_str = format_ulid(id);
    let mut stmt = conn
        .prepare(
            "SELECT title, exam_date, window_mode, window_weeks, reasons_json, created_ms, updated_ms
             FROM exam_plan WHERE id = ?1",
        )
        .map_err(sql_err)?;
    let mut rows = stmt
        .query_map([&id_str], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, u16>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })
        .map_err(sql_err)?;
    let Some(first) = rows.next() else {
        return Ok(None);
    };
    let (title, date_str, mode, weeks, reasons_json, created_ms, updated_ms) =
        first.map_err(sql_err)?;
    let exam_date = parse_date(&date_str).ok_or_else(|| corrupt("exam_date", &date_str))?;
    let window = decode_window(&mode, weeks, &reasons_json)?;
    let targets = load_targets(conn, &id_str)?;
    let blocks = load_blocks(conn, &id_str)?;
    Ok(Some(ExamPlan {
        id,
        title,
        exam_date,
        window,
        targets,
        blocks,
        created_ms,
        updated_ms,
    }))
}

/// List all `ExamPlan` records ordered by `exam_date` ascending.
pub fn list_exam_plans(conn: &Connection) -> Result<Vec<ExamPlan>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT id FROM exam_plan ORDER BY exam_date ASC, title ASC")
        .map_err(sql_err)?;
    let ids: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;
    let mut out = Vec::with_capacity(ids.len());
    for id_str in ids {
        let uid = parse_ulid(&id_str).ok_or_else(|| corrupt("exam_plan.id", &id_str))?;
        if let Some(plan) = load_exam_plan(conn, uid)? {
            out.push(plan);
        }
    }
    Ok(out)
}

/// Delete an `ExamPlan` and cascade-delete its targets and blocks.
pub fn delete_exam_plan(conn: &Connection, id: Uid) -> Result<bool, StoreError> {
    conn.execute("DELETE FROM exam_plan WHERE id = ?1", [format_ulid(id)])
        .map(|n| n > 0)
        .map_err(sql_err)
}
