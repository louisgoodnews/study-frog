//! Which reminders are due. Pure: the caller supplies the current local time and what has fired.

use studyfrog_core::civil::{Date, LocalTime, add_days, date_to_days};
use studyfrog_core::uid::Uid;

use crate::recurrence::occurrences_all;
use crate::session::Session;

/// One reminder that should be shown now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueReminder {
    /// The entry.
    pub session: Uid,
    /// Its title.
    pub title: String,
    /// The occurrence's local date.
    pub date: Date,
    /// Local start, minutes since midnight.
    pub start_minute: u16,
    /// The reminder's lead time in minutes.
    pub minutes_before: u32,
}

/// What has been shown already: (entry, occurrence date, lead time).
pub type Fired = (Uid, Date, u32);

fn absolute_minute(d: Date, minute: u16) -> i64 {
    date_to_days(d) * 1440 + i64::from(minute)
}

/// Reminders whose time has come and whose occurrence has not ended, minus those already fired. A
/// reminder that was missed while the app was closed still shows once, as long as the occurrence
/// has not finished.
pub fn due_reminders(sessions: &[Session], now: LocalTime, fired: &[Fired]) -> Vec<DueReminder> {
    let now_abs = absolute_minute(now.date, now.minute);
    let from = add_days(now.date, -1);
    let to = add_days(
        now.date,
        i64::from(crate::session::MAX_REMINDER_MIN / 1440) + 1,
    );
    let mut out = Vec::new();
    for o in occurrences_all(sessions, from, to) {
        let Some(s) = sessions.iter().find(|s| s.id == o.session) else {
            continue;
        };
        let start = absolute_minute(o.date, o.start_minute);
        let end = start + i64::from(o.duration_min);
        for &lead in &s.reminders {
            let trigger = start - i64::from(lead);
            let shown = fired.contains(&(s.id, o.date, lead));
            if !shown && trigger <= now_abs && now_abs < end {
                out.push(DueReminder {
                    session: s.id,
                    title: s.title.clone(),
                    date: o.date,
                    start_minute: o.start_minute,
                    minutes_before: lead,
                });
            }
        }
    }
    out
}
