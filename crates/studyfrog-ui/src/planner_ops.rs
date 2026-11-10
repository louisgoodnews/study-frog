//! Helper operations for the planner state (STORY-0141, STORY-0142).

use studyfrog_calendar::recurrence::occurrences_all;
use studyfrog_calendar::session::Session;
use studyfrog_core::civil::{Date, add_days, add_months};

use crate::calendar_mode::CalendarViewMode;
use crate::effect::Effect;
use crate::planner::{Planner, grid_of};
use crate::session_form::session_from_form;
use crate::state::{Notice, NoticeKind, State};
use crate::timetable_form::timetable_from_form;

/// Return error notice state.
pub fn failed(state: State, text: String) -> (State, Vec<Effect>) {
    let notice = Some(Notice {
        kind: NoticeKind::Error,
        text,
    });
    (State { notice, ..state }, Vec::new())
}

/// Transition state with a new planner value.
pub fn with(state: State, p: Planner) -> (State, Vec<Effect>) {
    (
        State {
            planner: p,
            ..state
        },
        Vec::new(),
    )
}

/// Replace sessions and update visible occurrences.
pub fn with_sessions(p: Planner, sessions: Vec<Session>) -> Planner {
    let grid = grid_of(p.month);
    let shown = occurrences_all(&sessions, grid[0], grid[41]);
    Planner {
        sessions,
        shown,
        ..p
    }
}

/// Extract year and month tuple from a civil date.
pub fn month_of(d: Date) -> (i32, u8) {
    (d.year, d.month)
}

/// Shift visible calendar month by delta months.
pub fn shift_month(p: Planner, delta: i64) -> Planner {
    let first = Date {
        year: p.month.0,
        month: p.month.1,
        day: 1,
    };
    let month = add_months(first, delta).map_or(p.month, month_of);
    let sessions = p.sessions.clone();
    with_sessions(Planner { month, ..p }, sessions)
}

/// Shift active date period based on the current view mode.
pub fn shift_period(p: Planner, delta: i64) -> Planner {
    match p.view_mode {
        CalendarViewMode::Month => shift_month(p, delta),
        CalendarViewMode::Week => {
            let selected = add_days(p.selected, delta * 7);
            let month = month_of(selected);
            let sessions = p.sessions.clone();
            with_sessions(
                Planner {
                    selected,
                    month,
                    ..p
                },
                sessions,
            )
        }
        CalendarViewMode::Day => {
            let selected = add_days(p.selected, delta);
            let month = month_of(selected);
            let sessions = p.sessions.clone();
            with_sessions(
                Planner {
                    selected,
                    month,
                    ..p
                },
                sessions,
            )
        }
        CalendarViewMode::Timetable => p,
    }
}

/// Transition planner when sessions and today are loaded.
pub fn loaded(p: Planner, sessions: Vec<Session>, today: Date) -> Planner {
    let first = !p.loaded;
    let p = Planner {
        loaded: true,
        today,
        ..p
    };
    let p = if first {
        Planner {
            month: month_of(today),
            selected: today,
            ..p
        }
    } else {
        p
    };
    with_sessions(p, sessions)
}

/// Save session from form.
pub fn save(state: State) -> (State, Vec<Effect>) {
    let Some(form) = &state.planner.form else {
        return (state, Vec::new());
    };
    match session_from_form(form) {
        Ok(s) => (state, vec![Effect::SaveSession(s)]),
        Err(e) => {
            let planner = Planner {
                form_error: Some(e),
                ..state.planner.clone()
            };
            with(state, planner)
        }
    }
}

/// Save timetable entry from form.
pub fn save_timetable(state: State) -> (State, Vec<Effect>) {
    let Some(form) = &state.planner.timetable_form else {
        return (state, Vec::new());
    };
    match timetable_from_form(form) {
        Ok(entry) => (state, vec![Effect::SaveTimetable(entry)]),
        Err(e) => {
            let planner = Planner {
                timetable_error: Some(e),
                ..state.planner.clone()
            };
            with(state, planner)
        }
    }
}

/// Handle result of sessions save or delete.
pub fn changed(state: State, result: Result<Vec<Session>, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(sessions) => {
            let p = Planner {
                form: None,
                form_error: None,
                ..state.planner.clone()
            };
            with(state, with_sessions(p, sessions))
        }
        Err(e) if state.planner.form.is_some() => {
            let planner = Planner {
                form_error: Some(e),
                ..state.planner.clone()
            };
            with(state, planner)
        }
        Err(e) => failed(state, e),
    }
}
