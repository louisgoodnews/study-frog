//! Reminders that are due now, calendar entries and homework deadlines alike, each shown once.

use studyfrog_calendar::reminders::{DueAll, due_all};
use studyfrog_core::civil::{add_days, utc_to_local};
use studyfrog_core::time::UnixMs;
use studyfrog_store::homework::{fired_homework_reminders, mark_homework_reminder_fired};
use studyfrog_store::sessions::{fired_reminders, load_sessions, mark_reminder_fired};

use crate::homework::list_homework;
use crate::session::{ServiceError, VaultSession};

/// The reminders due at `now`, read without marking them shown (for `reminders due`).
pub fn due_now(
    session: &VaultSession,
    now: UnixMs,
    offset_minutes: i32,
) -> Result<DueAll, ServiceError> {
    let local = utc_to_local(now, offset_minutes);
    let sessions = load_sessions(&session.conn).map_err(ServiceError::Store)?;
    let fired =
        fired_reminders(&session.conn, add_days(local.date, -2)).map_err(ServiceError::Store)?;
    let items = list_homework(session)?;
    let fired_homework = fired_homework_reminders(&session.conn).map_err(ServiceError::Store)?;
    Ok(due_all(&sessions, &items, local, &fired, &fired_homework))
}

/// The reminders due at `now`, calendar and homework; each is marked as shown, so the next call does
/// not return it again.
pub fn take_due_reminders(
    session: &VaultSession,
    now: UnixMs,
    offset_minutes: i32,
) -> Result<DueAll, ServiceError> {
    let due = due_now(session, now, offset_minutes)?;
    for d in &due.sessions {
        mark_reminder_fired(&session.conn, d.session, d.date, d.minutes_before)
            .map_err(ServiceError::Store)?;
    }
    for h in &due.homework {
        mark_homework_reminder_fired(&session.conn, h.item, h.lead_min)
            .map_err(ServiceError::Store)?;
    }
    Ok(due)
}
