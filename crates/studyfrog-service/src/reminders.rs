//! Calendar reminders that are due now, each shown once.

use studyfrog_calendar::reminders::{DueReminder, due_reminders};
use studyfrog_core::civil::{add_days, utc_to_local};
use studyfrog_core::time::UnixMs;
use studyfrog_store::sessions::{fired_reminders, load_sessions, mark_reminder_fired};

use crate::session::{ServiceError, VaultSession};

/// The reminders whose time has come at `now`; they are marked as shown, so the next call does not
/// return them again.
pub fn take_due_reminders(
    session: &VaultSession,
    now: UnixMs,
    offset_minutes: i32,
) -> Result<Vec<DueReminder>, ServiceError> {
    let local = utc_to_local(now, offset_minutes);
    let sessions = load_sessions(&session.conn).map_err(ServiceError::Store)?;
    let fired =
        fired_reminders(&session.conn, add_days(local.date, -2)).map_err(ServiceError::Store)?;
    let due = due_reminders(&sessions, local, &fired);
    for d in &due {
        mark_reminder_fired(&session.conn, d.session, d.date, d.minutes_before)
            .map_err(ServiceError::Store)?;
    }
    Ok(due)
}
