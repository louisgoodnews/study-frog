//! The answers of the calendar commands: the homework due on a day with its urgency, and the reminders
//! due now (read without marking them shown).

use serde_json::{Value, json};
use studyfrog_calendar::homework_query::{due_on, urgency};
use studyfrog_core::civil::{Date, format_date, format_minute};

use crate::control_cmd::calendar::SessionAdd;
use crate::control_state::ulid;
use crate::state::State;

/// The session `add` created: found by its title and start (the latest one when several match), or
/// the form's error when the save was refused (the form stays open).
pub fn session_added(state: &State, add: &SessionAdd) -> Result<Value, String> {
    if let Some(e) = &state.planner.form_error {
        return Err(format!("{e} (the new-session form is still open)"));
    }
    state
        .planner
        .sessions
        .iter()
        .filter(|s| s.title == add.title && s.start == add.start)
        .max_by_key(|s| s.id.0)
        .map(|s| {
            json!({
                "session": {
                    "id": ulid(s.id),
                    "title": s.title,
                    "start": format!("{} {}", format_date(s.start.date), format_minute(s.start.minute)),
                    "minutes": s.duration_min,
                    "reminders": s.reminders,
                }
            })
        })
        .ok_or_else(|| "the session was not saved".to_owned())
}

/// The homework due on `day`, earliest first, with each item's urgency (null until the clock is read).
pub fn homework_json(state: &State, day: Date) -> Value {
    let now = state.planner.homework_now;
    let items: Vec<Value> = due_on(&state.planner.homework, day)
        .into_iter()
        .map(|i| {
            json!({
                "title": i.title,
                "due": format!("{} {:02}:{:02}", format_date(i.due.date), i.due.minute / 60, i.due.minute % 60),
                "urgency": now.map(|n| format!("{:?}", urgency(i, n))),
            })
        })
        .collect();
    json!({"day": format_date(day), "homework": items})
}

/// The reminders due at the last `reminders due`; `loaded` is false until one was read.
pub fn reminders_json(state: &State) -> Value {
    match &state.planner.due_peek {
        None => json!({"loaded": false}),
        Some(due) => json!({
            "loaded": true,
            "sessions": due.sessions.iter().map(|r| json!({
                "title": r.title,
                "date": format_date(r.date),
                "minutes_before": r.minutes_before,
            })).collect::<Vec<_>>(),
            "homework": due.homework.iter().map(|h| json!({
                "title": h.title,
                "lead_min": h.lead_min,
            })).collect::<Vec<_>>(),
        }),
    }
}

fn view_name(mode: crate::calendar_mode::CalendarViewMode) -> &'static str {
    use crate::calendar_mode::CalendarViewMode;
    match mode {
        CalendarViewMode::Month => "month",
        CalendarViewMode::Week => "week",
        CalendarViewMode::Day => "day",
        CalendarViewMode::Timetable => "timetable",
    }
}

/// The selection on screen: the view, the visible month, the selected day and its sessions.
pub fn selection_json(state: &State) -> Value {
    let p = &state.planner;
    let (year, month) = p.month;
    let sessions: Vec<Value> = p
        .sessions
        .iter()
        .filter(|s| s.start.date == p.selected)
        .map(|s| {
            json!({"title": s.title, "start": format_minute(s.start.minute), "minutes": s.duration_min})
        })
        .collect();
    json!({
        "view": view_name(p.view_mode),
        "month": format!("{year:04}-{month:02}"),
        "selected": format_date(p.selected),
        "sessions": sessions,
    })
}
