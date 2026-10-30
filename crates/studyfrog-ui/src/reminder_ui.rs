//! Calendar reminders arriving while the app is open: a toast each, and a line in the notification
//! list. Pure.

use studyfrog_calendar::reminders::DueReminder;
use studyfrog_core::civil::format_minute;

use crate::effect::Effect;
use crate::modal_types::{Toast, ToastAction};
use crate::state::{Notice, NoticeKind, State};

/// How many ticks (about a second each) pass between two looks at the calendar.
pub const CHECK_EVERY: u32 = 30;

fn lead_text(minutes: u32) -> String {
    match minutes {
        0 => "now".to_owned(),
        1..=59 => format!("in {minutes} min"),
        60..=1439 => format!("in {} h", minutes / 60),
        _ => format!("in {} days", minutes / 1440),
    }
}

/// The words of one reminder: "Due: Exam prep, in 1 days, 2027-03-01 09:00".
pub fn reminder_text(r: &DueReminder) -> String {
    let date = studyfrog_core::civil::format_date(r.date);
    let at = format_minute(r.start_minute);
    format!("{} ({}, {date} {at})", r.title, lead_text(r.minutes_before))
}

/// Count ticks while a vault is open and ask for a calendar check on the first and every
/// [`CHECK_EVERY`]th one.
pub fn tick(state: State) -> (State, Vec<Effect>) {
    if state.vault.is_none() {
        return (state, Vec::new());
    }
    let due_now = state.reminder_ticks.is_multiple_of(CHECK_EVERY);
    let state = State {
        reminder_ticks: state.reminder_ticks.wrapping_add(1),
        ..state
    };
    (
        state,
        if due_now {
            vec![Effect::CheckReminders]
        } else {
            Vec::new()
        },
    )
}

/// Show what came due. A failure is shown too, so a broken calendar does not fail silently.
pub fn arrived(state: State, result: Result<Vec<DueReminder>, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(list) if list.is_empty() => (state, Vec::new()),
        Ok(list) => {
            let mut state = state;
            for r in &list {
                state.toasts.push(Toast {
                    title: "Reminder".to_owned(),
                    detail: reminder_text(r),
                    action: ToastAction::OpenCalendar,
                });
            }
            let notice = Some(Notice {
                kind: NoticeKind::Info,
                text: format!("Reminder: {}", reminder_text(&list[0])),
            });
            for r in list.iter().skip(1) {
                state.chrome.notifications.push(Notice {
                    kind: NoticeKind::Info,
                    text: format!("Reminder: {}", reminder_text(r)),
                });
            }
            (State { notice, ..state }, Vec::new())
        }
        Err(e) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Checking reminders failed: {e}"),
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}
