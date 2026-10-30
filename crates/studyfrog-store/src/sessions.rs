//! Persistence of calendar sessions.

use rusqlite::{Connection, Row, params};
use studyfrog_calendar::session::{Ends, Repeat, Session, event_kind_from_name, event_kind_name};
use studyfrog_core::civil::{LocalTime, format_date, parse_date};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

fn corrupt(what: &str, v: &str) -> StoreError {
    StoreError::Corrupt(format!("{what}: {v:?}"))
}

fn repeat_columns(r: Repeat) -> (&'static str, u16, u8) {
    match r {
        Repeat::Never => ("never", 1, 0),
        Repeat::Daily { every } => ("daily", every, 0),
        Repeat::Weekly { every, days } => ("weekly", every, days),
        Repeat::Monthly { every } => ("monthly", every, 0),
    }
}

fn ends_columns(e: Ends) -> (&'static str, Option<String>, Option<u32>) {
    match e {
        Ends::Forever => ("forever", None, None),
        Ends::On(d) => ("on", Some(format_date(d)), None),
        Ends::After(n) => ("after", None, Some(n)),
    }
}

/// Insert or replace a session (and its cancelled dates) in one transaction.
pub fn save_session(conn: &mut Connection, s: &Session) -> Result<(), StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    let (kind, every, days) = repeat_columns(s.repeat);
    let (ends, ends_date, ends_count) = ends_columns(s.ends);
    let id = format_ulid(s.id);
    tx.execute(
        "INSERT INTO session (id, title, stack_uid, start_date, start_minute, duration_min, repeat_kind, repeat_every,
                              repeat_days, ends_kind, ends_date, ends_count, notes, kind)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(id) DO UPDATE SET title = ?2, stack_uid = ?3, start_date = ?4, start_minute = ?5, duration_min = ?6,
             repeat_kind = ?7, repeat_every = ?8, repeat_days = ?9, ends_kind = ?10, ends_date = ?11, ends_count = ?12,
             notes = ?13, kind = ?14",
        params![
            id, s.title, s.stack.map(format_ulid), format_date(s.start.date), s.start.minute, s.duration_min, kind,
            every, days, ends, ends_date, ends_count, s.notes, event_kind_name(s.kind)
        ],
    )
    .map_err(sql_err)?;
    tx.execute("DELETE FROM session_cancelled WHERE session_id = ?1", [&id])
        .map_err(sql_err)?;
    for d in &s.cancelled {
        tx.execute(
            "INSERT OR IGNORE INTO session_cancelled (session_id, date) VALUES (?1, ?2)",
            params![id, format_date(*d)],
        )
        .map_err(sql_err)?;
    }
    tx.execute("DELETE FROM session_reminder WHERE session_id = ?1", [&id])
        .map_err(sql_err)?;
    for m in &s.reminders {
        tx.execute(
            "INSERT OR IGNORE INTO session_reminder (session_id, minutes_before) VALUES (?1, ?2)",
            params![id, m],
        )
        .map_err(sql_err)?;
    }
    tx.commit().map_err(sql_err)
}

/// Delete a session; returns whether it existed.
pub fn delete_session(conn: &Connection, id: Uid) -> Result<bool, StoreError> {
    conn.execute("DELETE FROM session WHERE id = ?1", [format_ulid(id)])
        .map(|n| n > 0)
        .map_err(sql_err)
}

struct Raw {
    id: String,
    title: String,
    stack: Option<String>,
    date: String,
    minute: u16,
    duration: u16,
    kind: String,
    every: u16,
    days: u8,
    ends: String,
    ends_date: Option<String>,
    ends_count: Option<u32>,
    notes: String,
    entry_kind: String,
}

fn raw(r: &Row) -> rusqlite::Result<Raw> {
    Ok(Raw {
        id: r.get(0)?,
        title: r.get(1)?,
        stack: r.get(2)?,
        date: r.get(3)?,
        minute: r.get(4)?,
        duration: r.get(5)?,
        kind: r.get(6)?,
        every: r.get(7)?,
        days: r.get(8)?,
        ends: r.get(9)?,
        ends_date: r.get(10)?,
        ends_count: r.get(11)?,
        notes: r.get(12)?,
        entry_kind: r.get(13)?,
    })
}

fn session_of(
    r: Raw,
    cancelled: Vec<studyfrog_core::civil::Date>,
    reminders: Vec<u32>,
) -> Result<Session, StoreError> {
    let repeat = match r.kind.as_str() {
        "never" => Repeat::Never,
        "daily" => Repeat::Daily { every: r.every },
        "weekly" => Repeat::Weekly {
            every: r.every,
            days: r.days,
        },
        "monthly" => Repeat::Monthly { every: r.every },
        other => return Err(corrupt("repeat_kind", other)),
    };
    let ends = match (r.ends.as_str(), &r.ends_date, r.ends_count) {
        ("forever", _, _) => Ends::Forever,
        ("on", Some(d), _) => Ends::On(parse_date(d).ok_or_else(|| corrupt("ends_date", d))?),
        ("after", _, Some(n)) => Ends::After(n),
        _ => return Err(corrupt("ends", &r.ends)),
    };
    Ok(Session {
        id: parse_ulid(&r.id).ok_or_else(|| corrupt("ulid", &r.id))?,
        title: r.title,
        stack: r
            .stack
            .as_deref()
            .map(|s| parse_ulid(s).ok_or_else(|| corrupt("ulid", s)))
            .transpose()?,
        start: LocalTime {
            date: parse_date(&r.date).ok_or_else(|| corrupt("start_date", &r.date))?,
            minute: r.minute,
        },
        duration_min: r.duration,
        repeat,
        ends,
        cancelled,
        notes: r.notes,
        kind: event_kind_from_name(&r.entry_kind).ok_or_else(|| corrupt("kind", &r.entry_kind))?,
        reminders,
    })
}

fn cancelled_of(
    conn: &Connection,
    id: &str,
) -> Result<Vec<studyfrog_core::civil::Date>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT date FROM session_cancelled WHERE session_id = ?1 ORDER BY date")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([id], |r| r.get::<_, String>(0))
        .map_err(sql_err)?;
    rows.map(|d| {
        let d = d.map_err(sql_err)?;
        parse_date(&d).ok_or_else(|| corrupt("cancelled date", &d))
    })
    .collect()
}

fn reminders_of(conn: &Connection, id: &str) -> Result<Vec<u32>, StoreError> {
    let sql =
        "SELECT minutes_before FROM session_reminder WHERE session_id = ?1 ORDER BY minutes_before";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([id], |r| r.get::<_, u32>(0))
        .map_err(sql_err)?;
    rows.collect::<Result<_, _>>().map_err(sql_err)
}

/// Remember that this reminder was shown for this occurrence.
pub fn mark_reminder_fired(
    conn: &Connection,
    id: Uid,
    date: studyfrog_core::civil::Date,
    minutes_before: u32,
) -> Result<(), StoreError> {
    let sql = "INSERT OR IGNORE INTO reminder_fired (session_id, date, minutes_before) VALUES (?1, ?2, ?3)";
    conn.execute(
        sql,
        params![format_ulid(id), format_date(date), minutes_before],
    )
    .map(|_| ())
    .map_err(sql_err)
}

/// Every reminder shown for occurrences on or after `since`.
pub fn fired_reminders(
    conn: &Connection,
    since: studyfrog_core::civil::Date,
) -> Result<Vec<(Uid, studyfrog_core::civil::Date, u32)>, StoreError> {
    let sql = "SELECT session_id, date, minutes_before FROM reminder_fired WHERE date >= ?1";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([format_date(since)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, u32>(2)?,
            ))
        })
        .map_err(sql_err)?;
    rows.map(|row| {
        let (id, d, m) = row.map_err(sql_err)?;
        Ok((
            parse_ulid(&id).ok_or_else(|| corrupt("ulid", &id))?,
            parse_date(&d).ok_or_else(|| corrupt("date", &d))?,
            m,
        ))
    })
    .collect()
}

/// Every session, ordered by start.
pub fn load_sessions(conn: &Connection) -> Result<Vec<Session>, StoreError> {
    let sql = "SELECT id, title, stack_uid, start_date, start_minute, duration_min, repeat_kind, repeat_every, repeat_days,
                      ends_kind, ends_date, ends_count, notes, kind FROM session ORDER BY start_date, start_minute, id";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows: Vec<Raw> = stmt
        .query_map([], raw)
        .map_err(sql_err)?
        .collect::<Result<_, _>>()
        .map_err(sql_err)?;
    rows.into_iter()
        .map(|r| {
            let cancelled = cancelled_of(conn, &r.id)?;
            let reminders = reminders_of(conn, &r.id)?;
            session_of(r, cancelled, reminders)
        })
        .collect()
}
