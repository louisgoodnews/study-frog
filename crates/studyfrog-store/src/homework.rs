//! SQLite CRUD persistence for homework items, checklists, and reminders.

use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};
use studyfrog_calendar::homework::{
    ChecklistItem, HomeworkItem, homework_priority_from_name, homework_priority_name,
    homework_status_from_name, homework_status_name, normalize_reminders,
};
use studyfrog_core::civil::{LocalTime, format_date, parse_date};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

fn corrupt(what: &str, value: &str) -> StoreError {
    StoreError::Corrupt(format!("{what}: {value:?}"))
}

struct RawHomework {
    id: String,
    title: String,
    subject: Option<String>,
    stack_uid: Option<String>,
    timetable_id: Option<String>,
    due_date: String,
    due_minute: u16,
    start_date: Option<String>,
    estimated_minutes: u32,
    priority: String,
    status: String,
    notes: String,
}

fn row_to_raw(r: &Row) -> rusqlite::Result<RawHomework> {
    Ok(RawHomework {
        id: r.get(0)?,
        title: r.get(1)?,
        subject: r.get(2)?,
        stack_uid: r.get(3)?,
        timetable_id: r.get(4)?,
        due_date: r.get(5)?,
        due_minute: r.get(6)?,
        start_date: r.get(7)?,
        estimated_minutes: r.get(8)?,
        priority: r.get(9)?,
        status: r.get(10)?,
        notes: r.get(11)?,
    })
}

fn opt_uid(raw: Option<String>) -> Result<Option<Uid>, StoreError> {
    raw.as_deref()
        .map(|s| parse_ulid(s).ok_or_else(|| corrupt("ulid", s)))
        .transpose()
}

fn load_checklist(conn: &Connection, id: &str) -> Result<Vec<ChecklistItem>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT text, done FROM homework_checklist WHERE homework_id = ?1 ORDER BY pos")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([id], |r| {
            Ok(ChecklistItem {
                text: r.get(0)?,
                done: r.get::<_, i64>(1)? != 0,
            })
        })
        .map_err(sql_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_err)
}

fn load_reminders(conn: &Connection, id: &str) -> Result<Vec<u32>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT minutes_before FROM homework_reminder WHERE homework_id = ?1 ORDER BY minutes_before",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([id], |r| r.get::<_, u32>(0))
        .map_err(sql_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_err)
}

fn decode_row(conn: &Connection, raw: RawHomework) -> Result<HomeworkItem, StoreError> {
    let checklist = load_checklist(conn, &raw.id)?;
    let reminders = load_reminders(conn, &raw.id)?;
    let id = parse_ulid(&raw.id).ok_or_else(|| corrupt("ulid", &raw.id))?;
    let due_date = parse_date(&raw.due_date).ok_or_else(|| corrupt("due_date", &raw.due_date))?;
    let start_date = raw
        .start_date
        .as_deref()
        .map(|s| parse_date(s).ok_or_else(|| corrupt("start_date", s)))
        .transpose()?;
    let priority = homework_priority_from_name(&raw.priority)
        .ok_or_else(|| corrupt("priority", &raw.priority))?;
    let status =
        homework_status_from_name(&raw.status).ok_or_else(|| corrupt("status", &raw.status))?;
    Ok(HomeworkItem {
        id,
        title: raw.title,
        subject: raw.subject,
        stack_uid: opt_uid(raw.stack_uid)?,
        timetable_id: opt_uid(raw.timetable_id)?,
        due: LocalTime {
            date: due_date,
            minute: raw.due_minute,
        },
        start_date,
        estimated_minutes: raw.estimated_minutes,
        priority,
        status,
        checklist,
        notes: raw.notes,
        reminders,
    })
}

/// Load all homework items ordered by due date, due minute, and id.
pub fn list_homework(conn: &Connection) -> Result<Vec<HomeworkItem>, StoreError> {
    let sql = "SELECT id, title, subject, stack_uid, timetable_id, due_date, due_minute,
                      start_date, estimated_minutes, priority, status, notes
               FROM homework ORDER BY due_date, due_minute, id";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let raw_rows: Vec<RawHomework> = stmt
        .query_map([], row_to_raw)
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;
    raw_rows.into_iter().map(|r| decode_row(conn, r)).collect()
}

/// Load a single homework item by id, returning `None` if absent.
pub fn get_homework(conn: &Connection, id: Uid) -> Result<Option<HomeworkItem>, StoreError> {
    let sql = "SELECT id, title, subject, stack_uid, timetable_id, due_date, due_minute,
                      start_date, estimated_minutes, priority, status, notes
               FROM homework WHERE id = ?1";
    let raw = conn
        .query_row(sql, [format_ulid(id)], row_to_raw)
        .optional()
        .map_err(sql_err)?;
    raw.map(|r| decode_row(conn, r)).transpose()
}

/// Upsert one homework item inside an existing SQLite transaction without wiping fired history.
pub fn upsert_homework_tx(tx: &Transaction<'_>, item: &HomeworkItem) -> Result<(), StoreError> {
    let id = format_ulid(item.id);
    tx.execute(
        "INSERT INTO homework (id, title, subject, stack_uid, timetable_id, due_date, due_minute,
                               start_date, estimated_minutes, priority, status, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(id) DO UPDATE SET title = ?2, subject = ?3, stack_uid = ?4, timetable_id = ?5,
             due_date = ?6, due_minute = ?7, start_date = ?8, estimated_minutes = ?9,
             priority = ?10, status = ?11, notes = ?12",
        params![
            id,
            item.title,
            item.subject,
            item.stack_uid.map(format_ulid),
            item.timetable_id.map(format_ulid),
            format_date(item.due.date),
            item.due.minute,
            item.start_date.map(format_date),
            item.estimated_minutes,
            homework_priority_name(item.priority),
            homework_status_name(item.status),
            item.notes,
        ],
    )
    .map_err(sql_err)?;
    tx.execute(
        "DELETE FROM homework_checklist WHERE homework_id = ?1",
        [&id],
    )
    .map_err(sql_err)?;
    for (pos, entry) in item.checklist.iter().enumerate() {
        let done = i64::from(entry.done);
        tx.execute(
            "INSERT INTO homework_checklist (homework_id, pos, text, done) VALUES (?1, ?2, ?3, ?4)",
            params![id, pos as i64, entry.text, done],
        )
        .map_err(sql_err)?;
    }
    tx.execute(
        "DELETE FROM homework_reminder WHERE homework_id = ?1",
        [&id],
    )
    .map_err(sql_err)?;
    for minutes in normalize_reminders(&item.reminders) {
        tx.execute(
            "INSERT OR IGNORE INTO homework_reminder (homework_id, minutes_before) VALUES (?1, ?2)",
            params![id, minutes],
        )
        .map_err(sql_err)?;
    }
    Ok(())
}

/// Insert or update a homework item in its own transaction, preserving fired reminder rows.
pub fn upsert_homework(conn: &mut Connection, item: &HomeworkItem) -> Result<(), StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    upsert_homework_tx(&tx, item)?;
    tx.commit().map_err(sql_err)
}

/// Delete a homework item by id; returns `true` when a row was removed.
pub fn delete_homework(conn: &Connection, id: Uid) -> Result<bool, StoreError> {
    conn.execute("DELETE FROM homework WHERE id = ?1", [format_ulid(id)])
        .map(|n| n > 0)
        .map_err(sql_err)
}
