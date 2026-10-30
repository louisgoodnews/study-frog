//! The calendar screen: state and update logic. Pure; recurrence comes from `studyfrog-calendar`
//! and is expanded here when the month or the sessions change, never per frame.

use studyfrog_calendar::recurrence::occurrences_all;
use studyfrog_calendar::session::{Occurrence, Session};
use studyfrog_core::civil::{Date, Weekday, add_months, month_grid};
use studyfrog_core::uid::Uid;

use crate::effect::Effect;
use crate::session_form::{SessionForm, apply_form_edit, form_from, form_new, session_from_form};

pub use crate::plan_msg::PlanMsg;
use crate::setup::{Preset, Rehearse, SetupEdit, apply_edit, form_for_preset, refresh_preview};
use crate::state::{Notice, NoticeKind, Screen, State};

/// The week starts on Monday.
pub const WEEK_START: Weekday = Weekday::Mon;

/// What the calendar needs to know.
#[derive(Debug, Clone, PartialEq)]
pub struct Planner {
    /// Sessions have been loaded at least once.
    pub loaded: bool,
    /// Today's local date as of the last load.
    pub today: Date,
    /// The visible month.
    pub month: (i32, u8),
    /// The day whose sessions the side panel lists.
    pub selected: Date,
    /// Every stored session.
    pub sessions: Vec<Session>,
    /// Occurrences inside the visible grid (six weeks).
    pub shown: Vec<Occurrence>,
    /// The session form, when open.
    pub form: Option<SessionForm>,
    /// Why the last save was refused.
    pub form_error: Option<String>,
}

impl Default for Planner {
    fn default() -> Self {
        let epoch = Date {
            year: 1970,
            month: 1,
            day: 1,
        };
        Planner {
            loaded: false,
            today: epoch,
            month: (1970, 1),
            selected: epoch,
            sessions: Vec::new(),
            shown: Vec::new(),
            form: None,
            form_error: None,
        }
    }
}

/// The six-week grid of the visible month.
pub fn grid_of(month: (i32, u8)) -> [Date; 42] {
    month_grid(month.0, month.1, WEEK_START)
}

fn failed(state: State, text: String) -> (State, Vec<Effect>) {
    let notice = Some(Notice {
        kind: NoticeKind::Error,
        text,
    });
    (State { notice, ..state }, Vec::new())
}

fn with(state: State, p: Planner) -> (State, Vec<Effect>) {
    (
        State {
            planner: p,
            ..state
        },
        Vec::new(),
    )
}

/// Replace the sessions and expand the visible grid.
fn with_sessions(p: Planner, sessions: Vec<Session>) -> Planner {
    let grid = grid_of(p.month);
    let shown = occurrences_all(&sessions, grid[0], grid[41]);
    Planner {
        sessions,
        shown,
        ..p
    }
}

fn month_of(d: Date) -> (i32, u8) {
    (d.year, d.month)
}

fn shift_month(p: Planner, delta: i64) -> Planner {
    let first = Date {
        year: p.month.0,
        month: p.month.1,
        day: 1,
    };
    let month = add_months(first, delta).map_or(p.month, month_of);
    let sessions = p.sessions.clone();
    with_sessions(Planner { month, ..p }, sessions)
}

fn loaded(p: Planner, sessions: Vec<Session>, today: Date) -> Planner {
    let first = !p.loaded;
    let p = Planner {
        loaded: true,
        today,
        ..p
    };
    if first {
        let p = Planner {
            month: month_of(today),
            selected: today,
            ..p
        };
        return with_sessions(p, sessions);
    }
    with_sessions(p, sessions)
}

fn save(state: State) -> (State, Vec<Effect>) {
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

fn changed(state: State, result: Result<Vec<Session>, String>) -> (State, Vec<Effect>) {
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

/// Open the rehearse screen with the session's stack as the only stack filter.
fn rehearse(state: State, id: Uid) -> (State, Vec<Effect>) {
    let stack = state.planner.sessions.iter().find(|s| s.id == id);
    let Some(s) = stack else {
        return (state, Vec::new());
    };
    let mut form = form_for_preset(Preset::Spaced, state.rehearse.form.mode);
    if let Some(uid) = s.stack {
        form = apply_edit(form, SetupEdit::Stack(uid, true));
    }
    refresh_preview(State {
        screen: Screen::Rehearse,
        rehearse: Rehearse {
            form,
            run: None,
            ..state.rehearse.clone()
        },
        ..state
    })
}

/// Handle the calendar messages.
pub fn update(state: State, msg: PlanMsg) -> (State, Vec<Effect>) {
    let p = state.planner.clone();
    match msg {
        PlanMsg::Loaded(Ok((sessions, today))) => with(state, loaded(p, sessions, today)),
        PlanMsg::Loaded(Err(e)) => failed(state, e),
        PlanMsg::Month(delta) => with(state, shift_month(p, delta)),
        PlanMsg::Today => {
            let sessions = p.sessions.clone();
            let p = Planner {
                month: month_of(p.today),
                selected: p.today,
                ..p
            };
            with(state, with_sessions(p, sessions))
        }
        PlanMsg::Select(selected) => with(state, Planner { selected, ..p }),
        PlanMsg::New => with(
            state,
            Planner {
                form: Some(form_new(p.selected)),
                form_error: None,
                ..p
            },
        ),
        PlanMsg::Edit(id) => {
            let form = p.sessions.iter().find(|s| s.id == id).map(form_from);
            with(state, Planner { form, ..p })
        }
        PlanMsg::CloseForm => with(
            state,
            Planner {
                form: None,
                form_error: None,
                ..p
            },
        ),
        PlanMsg::Form(edit) => {
            let form = p.form.clone().map(|f| apply_form_edit(f, edit));
            with(state, Planner { form, ..p })
        }
        PlanMsg::Save => save(state),
        PlanMsg::Delete(id) => (state, vec![Effect::DeleteSession(id)]),
        PlanMsg::Skip(id, date) => (state, vec![Effect::CancelOccurrence(id, date)]),
        PlanMsg::Changed(result) => changed(state, result),
        PlanMsg::Rehearse(id) => rehearse(state, id),
    }
}
