//! Persistence of timetable entries.

use rusqlite::{Connection, Row, params};
use studyfrog_calendar::timetable::TimetableEntry;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

fn corrupt(what: &str, v: &str) -> StoreError {
    StoreError::Corrupt(format!("{what}: {v:?}"))
}

/// Insert or replace a timetable entry.
pub fn save_timetable_entry(conn: &Connection, entry: &TimetableEntry) -> Result<(), StoreError> {
    let id = format_ulid(entry.id);
    let stack_uid = entry.stack_uid.map(format_ulid);
    conn.execute(
        "INSERT INTO timetable_entry (id, title, day_of_week, start_minute, duration_min, location, instructor, stack_uid)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET title = ?2, day_of_week = ?3, start_minute = ?4, duration_min = ?5,
             location = ?6, instructor = ?7, stack_uid = ?8",
        params![
            id,
            entry.title,
            entry.day_of_week,
            entry.start_minute,
            entry.duration_min,
            entry.location,
            entry.instructor,
            stack_uid,
        ],
    )
    .map_err(sql_err)?;
    Ok(())
}

/// Delete a timetable entry by id.
pub fn delete_timetable_entry(conn: &Connection, id: Uid) -> Result<bool, StoreError> {
    conn.execute(
        "DELETE FROM timetable_entry WHERE id = ?1",
        [format_ulid(id)],
    )
    .map(|n| n > 0)
    .map_err(sql_err)
}

type RawRow = (
    String,
    String,
    u8,
    u16,
    u16,
    Option<String>,
    Option<String>,
    Option<String>,
);

fn row_to_entry(r: &Row) -> rusqlite::Result<RawRow> {
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

/// Load all timetable entries, ordered by day of week then start time.
pub fn load_timetable_entries(conn: &Connection) -> Result<Vec<TimetableEntry>, StoreError> {
    let sql =
        "SELECT id, title, day_of_week, start_minute, duration_min, location, instructor, stack_uid
               FROM timetable_entry ORDER BY day_of_week, start_minute, id";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let raw_rows = stmt
        .query_map([], row_to_entry)
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;
    let mut entries = Vec::with_capacity(raw_rows.len());
    for (id_str, title, day_of_week, start_minute, duration_min, location, instructor, stack_str) in
        raw_rows
    {
        let id = parse_ulid(&id_str).ok_or_else(|| corrupt("ulid", &id_str))?;
        let stack_uid = stack_str
            .as_deref()
            .map(|s| parse_ulid(s).ok_or_else(|| corrupt("ulid", s)))
            .transpose()?;
        entries.push(TimetableEntry {
            id,
            title,
            day_of_week,
            start_minute,
            duration_min,
            location,
            instructor,
            stack_uid,
        });
    }
    Ok(entries)
}
