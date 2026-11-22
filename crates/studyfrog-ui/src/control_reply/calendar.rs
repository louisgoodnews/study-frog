//! The answers of the calendar commands: the homework due on a day with its urgency, and the reminders
//! due now (read without marking them shown).

use serde_json::{Value, json};
use studyfrog_calendar::homework_query::{due_on, urgency};
use studyfrog_core::civil::{Date, format_date};

use crate::state::State;

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
