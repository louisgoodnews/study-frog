//! Calendar and homework reminders that are due now, each shown once.

use rusqlite::{Connection, params};
use studyfrog_calendar::homework::{HomeworkItem, due_homework_reminders};
use studyfrog_calendar::reminders::{DueReminder, Fired, due_reminders};
use studyfrog_core::civil::{Date, LocalTime, add_days, format_date, parse_date, utc_to_local};
use studyfrog_core::time::UnixMs;
use studyfrog_md::ulid::{format_ulid, parse_ulid};
use studyfrog_store::error::{StoreError, sql_err};
use studyfrog_store::homework::list_homework;
use studyfrog_store::sessions::{fired_reminders, load_sessions, mark_reminder_fired};

use crate::session::{ServiceError, VaultSession};

/// The reminders whose time has come at `now`; they are marked as shown, so the next call does not
/// return them again. Session reminders come first, then homework reminders.
pub fn take_due_reminders(
    session: &VaultSession,
    now: UnixMs,
    offset_minutes: i32,
) -> Result<Vec<DueReminder>, ServiceError> {
    let local = utc_to_local(now, offset_minutes);
    let since = add_days(local.date, -2);
    let sessions = load_sessions(&session.conn).map_err(ServiceError::Store)?;
    let fired = fired_reminders(&session.conn, since).map_err(ServiceError::Store)?;
    let due = due_reminders(&sessions, local, &fired);
    for d in &due {
        mark_reminder_fired(&session.conn, d.session, d.date, d.minutes_before)
            .map_err(ServiceError::Store)?;
    }
    let homework = list_homework(&session.conn).map_err(ServiceError::Store)?;
    let homework_due = homework_due_now(&session.conn, &homework, local, since)?;
    let mut out = due;
    out.extend(homework_due);
    Ok(out)
}

/// The homework reminders due at `local`, each recorded in `homework_reminder_fired` before return.
fn homework_due_now(
    conn: &Connection,
    homework: &[HomeworkItem],
    local: LocalTime,
    since: Date,
) -> Result<Vec<DueReminder>, ServiceError> {
    let fired = homework_fired_since(conn, since).map_err(ServiceError::Store)?;
    let due = due_homework_reminders(homework, local, &fired);
    for d in &due {
        mark_homework_fired(conn, d).map_err(ServiceError::Store)?;
    }
    Ok(due)
}

/// Homework reminders already shown for due dates on or after `since`.
fn homework_fired_since(conn: &Connection, since: Date) -> Result<Vec<Fired>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT homework_id, due_date, minutes_before FROM homework_reminder_fired
             WHERE due_date >= ?1",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([format_date(since)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, u32>(2)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (id, date, lead) = row.map_err(sql_err)?;
        let id = parse_ulid(&id).ok_or_else(|| StoreError::Corrupt(format!("homework id {id}")))?;
        let date =
            parse_date(&date).ok_or_else(|| StoreError::Corrupt(format!("due date {date}")))?;
        out.push((id, date, lead));
    }
    Ok(out)
}

/// Record one shown homework reminder. `INSERT OR IGNORE` keeps a repeated call harmless.
fn mark_homework_fired(conn: &Connection, d: &DueReminder) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR IGNORE INTO homework_reminder_fired (homework_id, due_date, minutes_before)
         VALUES (?1, ?2, ?3)",
        params![
            format_ulid(d.session),
            format_date(d.date),
            d.minutes_before
        ],
    )
    .map(|_| ())
    .map_err(sql_err)
}
