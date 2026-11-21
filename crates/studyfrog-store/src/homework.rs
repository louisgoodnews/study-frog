//! Persistence of homework items and their checklist steps (STORY-0165). A save replaces the item
//! and its steps in one transaction.

use std::collections::HashMap;

use rusqlite::{Connection, Row, params};
use studyfrog_calendar::homework::{ChecklistItem, HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, LocalTime, format_date, parse_date};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

fn corrupt(what: &str, v: &str) -> StoreError {
    StoreError::Corrupt(format!("{what}: {v:?}"))
}

pub(crate) fn priority_code(p: HomeworkPriority) -> &'static str {
    match p {
        HomeworkPriority::Low => "low",
        HomeworkPriority::Medium => "medium",
        HomeworkPriority::High => "high",
    }
}

pub(crate) fn priority_from_code(code: &str) -> Result<HomeworkPriority, StoreError> {
    match code {
        "low" => Ok(HomeworkPriority::Low),
        "medium" => Ok(HomeworkPriority::Medium),
        "high" => Ok(HomeworkPriority::High),
        other => Err(corrupt("priority", other)),
    }
}

pub(crate) fn status_code(s: HomeworkStatus) -> &'static str {
    match s {
        HomeworkStatus::NotStarted => "not_started",
        HomeworkStatus::InProgress => "in_progress",
        HomeworkStatus::Completed => "completed",
    }
}

pub(crate) fn status_from_code(code: &str) -> Result<HomeworkStatus, StoreError> {
    match code {
        "not_started" => Ok(HomeworkStatus::NotStarted),
        "in_progress" => Ok(HomeworkStatus::InProgress),
        "completed" => Ok(HomeworkStatus::Completed),
        other => Err(corrupt("status", other)),
    }
}

/// Insert or replace a homework item with its checklist.
pub fn save_homework(conn: &Connection, item: &HomeworkItem) -> Result<(), StoreError> {
    let id = format_ulid(item.id);
    let course = item.course.map(format_ulid);
    let start = item.start.map(format_date);
    let tx = conn.unchecked_transaction().map_err(sql_err)?;
    tx.execute(
        "INSERT INTO homework_item (id, title, course, start_date, due_date, due_minute,
             estimate_min, priority, status, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET title = ?2, course = ?3, start_date = ?4, due_date = ?5,
             due_minute = ?6, estimate_min = ?7, priority = ?8, status = ?9, notes = ?10",
        params![
            id,
            item.title,
            course,
            start,
            format_date(item.due.date),
            item.due.minute,
            item.estimate_min,
            priority_code(item.priority),
            status_code(item.status),
            item.notes,
        ],
    )
    .map_err(sql_err)?;
    tx.execute("DELETE FROM homework_step WHERE homework_id = ?1", [&id])
        .map_err(sql_err)?;
    for (position, step) in item.checklist.iter().enumerate() {
        tx.execute(
            "INSERT INTO homework_step (homework_id, position, text, done) VALUES (?1, ?2, ?3, ?4)",
            params![id, position as i64, step.text, i64::from(step.done)],
        )
        .map_err(sql_err)?;
    }
    tx.commit().map_err(sql_err)
}

/// Delete a homework item and its steps.
pub fn delete_homework(conn: &Connection, id: Uid) -> Result<bool, StoreError> {
    let id = format_ulid(id);
    let tx = conn.unchecked_transaction().map_err(sql_err)?;
    tx.execute("DELETE FROM homework_step WHERE homework_id = ?1", [&id])
        .map_err(sql_err)?;
    tx.execute("DELETE FROM homework_fired WHERE homework_id = ?1", [&id])
        .map_err(sql_err)?;
    let removed = tx
        .execute("DELETE FROM homework_item WHERE id = ?1", [&id])
        .map_err(sql_err)?;
    tx.commit().map_err(sql_err)?;
    Ok(removed > 0)
}

type RawItem = (
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    u16,
    u32,
    String,
    String,
    String,
);

fn row_to_raw(r: &Row) -> rusqlite::Result<RawItem> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get(6)?,
        r.get(7)?,
        r.get(8)?,
        r.get(9)?,
    ))
}

fn parse_day(text: &str) -> Result<Date, StoreError> {
    parse_date(text).ok_or_else(|| corrupt("date", text))
}

/// Load all homework items ordered by due date and time, with their checklists.
pub fn load_homework(conn: &Connection) -> Result<Vec<HomeworkItem>, StoreError> {
    let sql = "SELECT id, title, course, start_date, due_date, due_minute, estimate_min,
                      priority, status, notes
               FROM homework_item ORDER BY due_date, due_minute, id";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let raw = stmt
        .query_map([], row_to_raw)
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;
    let steps = load_steps(conn)?;
    let mut items = Vec::with_capacity(raw.len());
    for (id, title, course, start, due_date, due_minute, estimate_min, priority, status, notes) in
        raw
    {
        let uid = parse_ulid(&id).ok_or_else(|| corrupt("ulid", &id))?;
        let course = course
            .as_deref()
            .map(|s| parse_ulid(s).ok_or_else(|| corrupt("ulid", s)))
            .transpose()?;
        let start = start.as_deref().map(parse_day).transpose()?;
        items.push(HomeworkItem {
            id: uid,
            title,
            course,
            start,
            due: LocalTime {
                date: parse_day(&due_date)?,
                minute: due_minute,
            },
            estimate_min,
            priority: priority_from_code(&priority)?,
            status: status_from_code(&status)?,
            checklist: steps.get(&id).cloned().unwrap_or_default(),
            notes,
        });
    }
    Ok(items)
}

fn load_steps(conn: &Connection) -> Result<HashMap<String, Vec<ChecklistItem>>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT homework_id, text, done FROM homework_step ORDER BY homework_id, position")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;
    let mut map: HashMap<String, Vec<ChecklistItem>> = HashMap::new();
    for (id, text, done) in rows {
        map.entry(id).or_default().push(ChecklistItem {
            text,
            done: done != 0,
        });
    }
    Ok(map)
}

/// Load one homework item by id, if it exists.
pub fn load_homework_item(conn: &Connection, id: Uid) -> Result<Option<HomeworkItem>, StoreError> {
    let wanted = format_ulid(id);
    Ok(load_homework(conn)?
        .into_iter()
        .find(|item| format_ulid(item.id) == wanted))
}

/// Every reminder already shown: (item, lead time in minutes).
pub fn fired_homework_reminders(conn: &Connection) -> Result<Vec<(Uid, u32)>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT homework_id, lead_min FROM homework_fired")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;
    rows.into_iter()
        .map(|(id, lead)| Ok((parse_ulid(&id).ok_or_else(|| corrupt("ulid", &id))?, lead)))
        .collect()
}

/// Record that a lead-time reminder was shown, so it is not shown again.
pub fn mark_homework_reminder_fired(
    conn: &Connection,
    id: Uid,
    lead_min: u32,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR IGNORE INTO homework_fired (homework_id, lead_min) VALUES (?1, ?2)",
        params![format_ulid(id), lead_min],
    )
    .map_err(sql_err)?;
    Ok(())
}
