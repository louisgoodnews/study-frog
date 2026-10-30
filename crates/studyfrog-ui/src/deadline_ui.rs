//! Setting or changing a stack's due date: it opens the calendar form on the stack's deadline entry
//! (or on a new one), so the date picker and the reminders are the same as anywhere else. Pure.

use studyfrog_calendar::session::{EventKind, Session};
use studyfrog_core::civil::{Date, add_days};
use studyfrog_core::uid::Uid;

use crate::effect::Effect;
use crate::nav::navigate;
use crate::planner::Planner;
use crate::session_form::{SessionForm, form_from, form_new};
use crate::state::{Screen, State};

/// How far ahead a new due date starts out.
const DEFAULT_DAYS_AHEAD: i64 = 7;

/// The deadline entry of a stack, if it has one.
pub fn deadline_of(sessions: &[Session], stack: Uid) -> Option<&Session> {
    sessions
        .iter()
        .find(|s| s.kind == EventKind::Deadline && s.stack == Some(stack))
}

fn blank(state: &State, stack: Uid, today: Date) -> SessionForm {
    let title = state
        .vault
        .iter()
        .flat_map(|v| v.overview.stacks.iter())
        .find(|s| s.uid == stack)
        .map_or_else(|| "Due".to_owned(), |s| format!("Due: {}", s.title));
    SessionForm {
        title,
        stack: Some(stack),
        kind: EventKind::Deadline,
        duration: "15".to_owned(),
        reminders: vec![1440],
        ..form_new(add_days(today, DEFAULT_DAYS_AHEAD))
    }
}

/// Go to the calendar with the stack's deadline in the form.
pub fn open(state: State, stack: Uid) -> (State, Vec<Effect>) {
    let today = state.home.as_ref().map_or(state.planner.today, |h| h.today);
    let form = match deadline_of(&state.planner.sessions, stack) {
        Some(s) => form_from(s),
        None => blank(&state, stack, today),
    };
    let planner = Planner {
        form: Some(form),
        form_error: None,
        ..state.planner.clone()
    };
    navigate(State { planner, ..state }, Screen::Calendar)
}
