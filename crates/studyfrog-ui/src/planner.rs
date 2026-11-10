//! The calendar screen: state and update logic (STORY-0141, STORY-0142).

use studyfrog_calendar::session::{Occurrence, Session};
use studyfrog_calendar::timetable::TimetableEntry;
use studyfrog_core::civil::{Date, Weekday, month_grid};

use crate::calendar_mode::CalendarViewMode;
use crate::effect::Effect;
pub use crate::plan_msg::PlanMsg;
use crate::planner_ops::{
    changed, failed, loaded, month_of, save, save_timetable, shift_month, shift_period, with,
    with_sessions,
};
use crate::session_form::{SessionForm, apply_form_edit, form_from, form_new};
use crate::state::State;
use crate::timetable_form::{TimetableForm, timetable_form_from, timetable_form_new};

/// The week starts on Monday.
pub const WEEK_START: Weekday = Weekday::Mon;

/// What the calendar needs to know.
#[derive(Debug, Clone, PartialEq)]
pub struct Planner {
    /// Sessions have been loaded at least once.
    pub loaded: bool,
    /// Active view mode (Month, Week, Day, Timetable).
    pub view_mode: CalendarViewMode,
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
    /// Stored weekly timetable entries.
    pub timetable: Vec<TimetableEntry>,
    /// Form for creating or editing a timetable entry.
    pub timetable_form: Option<TimetableForm>,
    /// Error message for the timetable form.
    pub timetable_error: Option<String>,
    /// The session form, when open.
    pub form: Option<SessionForm>,
    /// Why the last save was refused.
    pub form_error: Option<String>,
    /// The reviews the schedule planned for the visible grid.
    pub reviews: Vec<studyfrog_calendar::review_plan::PlanEntry>,
    /// The entry whose date is being chosen.
    pub moving: Option<crate::planner_reviews::MoveForm>,
    /// How to take back the last move of reviews.
    pub undo: Option<Vec<studyfrog_service::review_plan::Prior>>,
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
            view_mode: CalendarViewMode::Month,
            today: epoch,
            month: (1970, 1),
            selected: epoch,
            sessions: Vec::new(),
            shown: Vec::new(),
            timetable: Vec::new(),
            timetable_form: None,
            timetable_error: None,
            form: None,
            form_error: None,
            reviews: Vec::new(),
            moving: None,
            undo: None,
        }
    }
}

/// The six-week grid of the visible month.
pub fn grid_of(month: (i32, u8)) -> [Date; 42] {
    month_grid(month.0, month.1, WEEK_START)
}

/// Handle the calendar messages.
pub fn update(state: State, msg: PlanMsg) -> (State, Vec<Effect>) {
    let refresh = matches!(
        msg,
        PlanMsg::Loaded(Ok(_)) | PlanMsg::Month(_) | PlanMsg::ShiftPeriod(_) | PlanMsg::Today
    );
    let (state, mut effects) = update_one(state, msg);
    if refresh {
        effects.extend(crate::planner_reviews::reload(&state));
    }
    (state, effects)
}

fn update_one(state: State, msg: PlanMsg) -> (State, Vec<Effect>) {
    let p = state.planner.clone();
    match msg {
        PlanMsg::Loaded(Ok((sessions, today))) => with(state, loaded(p, sessions, today)),
        PlanMsg::Loaded(Err(e)) => failed(state, e),
        PlanMsg::SetViewMode(view_mode) => with(state, Planner { view_mode, ..p }),
        PlanMsg::ShiftPeriod(delta) => with(state, shift_period(p, delta)),
        PlanMsg::TimetableLoaded(Ok(timetable)) => with(state, Planner { timetable, ..p }),
        PlanMsg::TimetableLoaded(Err(e)) => failed(state, e),
        PlanMsg::OpenTimetableForm(entry) => {
            let form = entry
                .as_ref()
                .map(timetable_form_from)
                .unwrap_or_else(|| timetable_form_new(0));
            with(
                state,
                Planner {
                    timetable_form: Some(form),
                    timetable_error: None,
                    ..p
                },
            )
        }
        PlanMsg::CloseTimetableForm => with(
            state,
            Planner {
                timetable_form: None,
                timetable_error: None,
                ..p
            },
        ),
        PlanMsg::SaveTimetable(_) => save_timetable(state),
        PlanMsg::DeleteTimetable(id) => (state, vec![Effect::DeleteTimetable(id)]),
        PlanMsg::TimetableChanged(Ok(timetable)) => {
            let planner = Planner {
                timetable,
                timetable_form: None,
                timetable_error: None,
                ..p
            };
            (State { planner, ..state }, vec![Effect::LoadSessions])
        }
        PlanMsg::TimetableChanged(Err(e)) => failed(state, e),
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
        PlanMsg::StackDeadline(stack) => crate::deadline_ui::open(state, stack),
        PlanMsg::SetStackDueDate {
            stack,
            date,
            reminders,
        } => crate::deadline_ui::set_due_date(state, stack, date, reminders),
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
        PlanMsg::Rehearse(id) => crate::planner_rehearse::rehearse(state, id),
        PlanMsg::Reviews(m) => crate::planner_reviews::update(state, m),
    }
}
