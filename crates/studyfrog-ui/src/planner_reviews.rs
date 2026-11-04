//! The planned reviews on the calendar: loading them, moving an entry to another day (dragged or by date), and undo. Pure.

use studyfrog_calendar::review_plan::PlanEntry;
use studyfrog_core::civil::{date_from_days, date_to_days, format_date, parse_date};
use studyfrog_core::uid::Uid;
use studyfrog_service::review_plan::{Moved, Prior};

use crate::effect::Effect;
use crate::planner::{Planner, grid_of};
use crate::state::{Notice, NoticeKind, Screen, Selection, State};

/// The entry being moved with the date field of the side panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveForm {
    /// The stack of the entry (`None` for the inbox).
    pub stack: Option<Uid>,
    /// The day it is on now.
    pub day: i64,
    /// The date typed or picked.
    pub text: String,
}

/// What the worker is asked to do with the planned reviews.
#[derive(Debug, Clone, PartialEq)]
pub enum ReviewEffect {
    /// Read the plan between two local days (counted from 1970-01-01).
    Load(i64, i64),
    /// Make these items due on a local day in a stack (`None`: the Inbox).
    Move(Option<Uid>, Vec<Uid>, i64),
    /// Put the due dates of a move back.
    Undo(Vec<Prior>),
}

/// What happened to the planned reviews.
#[derive(Debug, Clone, PartialEq)]
pub enum ReviewMsg {
    /// The planned reviews were read.
    Loaded(Result<Vec<PlanEntry>, String>),
    /// Open the date field for an entry.
    OpenMove(Option<Uid>, i64),
    /// The date field was edited.
    MoveText(String),
    /// Move the entry to the typed date.
    Confirm,
    /// Close the date field.
    Cancel,
    /// An entry was dropped on another day.
    Drop {
        /// The stack of the entry.
        stack: Option<Uid>,
        /// The day it was on.
        from: i64,
        /// The day it was dropped on.
        to: i64,
    },
    /// The items were moved.
    Moved(Result<Box<Moved>, String>),
    /// Take the last move back.
    Undo,
    /// The last move was taken back.
    Undone(Result<(), String>),
    /// Open the stack of an entry in the library.
    OpenStack(Option<Uid>),
    /// Show or hide the planned reviews.
    Show(bool),
}

fn notice(state: &mut State, kind: NoticeKind, text: String) {
    state.notice = Some(Notice { kind, text });
}

/// Whether the calendar shows the planned reviews.
pub fn shown(state: &State) -> bool {
    !state.app.calendar_reviews_off
}

/// The effect that reads the reviews planned for the visible grid (nothing for a month wholly in the past).
pub fn reload(state: &State) -> Vec<Effect> {
    let p = &state.planner;
    let grid = grid_of(p.month);
    let (from, to) = (
        date_to_days(p.today).max(date_to_days(grid[0])),
        date_to_days(grid[41]),
    );
    if shown(state) && p.loaded && from <= to {
        vec![Effect::Reviews(ReviewEffect::Load(from, to))]
    } else {
        Vec::new()
    }
}

/// The items of the entry on `day` for `stack`.
fn items_of(p: &Planner, stack: Option<Uid>, day: i64) -> Vec<Uid> {
    p.reviews
        .iter()
        .find(|e| e.stack == stack && e.day == day)
        .map(|e| e.items.clone())
        .unwrap_or_default()
}

fn move_to(state: State, stack: Option<Uid>, from: i64, to: i64) -> (State, Vec<Effect>) {
    let items = items_of(&state.planner, stack, from);
    let today = date_to_days(state.planner.today);
    let mut state = state;
    if to < today {
        notice(
            &mut state,
            NoticeKind::Error,
            "Choose today or a later day.".to_owned(),
        );
        return (state, Vec::new());
    }
    if from == to || items.is_empty() {
        return (state, Vec::new());
    }
    (
        state,
        vec![Effect::Reviews(ReviewEffect::Move(stack, items, to))],
    )
}

fn confirm(state: State) -> (State, Vec<Effect>) {
    let Some(form) = state.planner.moving.clone() else {
        return (state, Vec::new());
    };
    match parse_date(&form.text) {
        Some(date) => move_to(state, form.stack, form.day, date_to_days(date)),
        None => {
            let mut state = state;
            notice(
                &mut state,
                NoticeKind::Error,
                "Type the date as 2026-11-20.".to_owned(),
            );
            (state, Vec::new())
        }
    }
}

fn moved(mut state: State, result: Result<Box<Moved>, String>) -> (State, Vec<Effect>) {
    state.planner.moving = None;
    match result {
        Ok(m) => {
            let warn = m
                .warning
                .as_ref()
                .map_or(String::new(), |w| format!(" {w}"));
            let text = format!("Moved the reviews of {} item(s).{warn}", m.moved);
            notice(&mut state, NoticeKind::Info, text);
            state.planner.undo = (!m.undo.is_empty()).then_some(m.undo);
            state.library.stale = true;
            let reload = reload(&state);
            (state, reload)
        }
        Err(e) => {
            notice(
                &mut state,
                NoticeKind::Error,
                format!("Could not move the reviews: {e}"),
            );
            (state, Vec::new())
        }
    }
}

fn open_stack(state: State, stack: Option<Uid>) -> (State, Vec<Effect>) {
    let folder = state.vault.as_ref().and_then(|v| {
        v.overview
            .stacks
            .iter()
            .find(|s| Some(s.uid) == stack)
            .map(|s| s.folder.clone())
    });
    let selection = folder.map_or(Selection::Inbox, Selection::Stack);
    let (state, mut effects) = crate::nav::navigate(state, Screen::Library);
    let (state, more) = crate::browse::update(state, crate::msg::Msg::Select(selection));
    effects.extend(more);
    (state, effects)
}

/// Handle a message about the planned reviews.
pub fn update(state: State, msg: ReviewMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    match msg {
        ReviewMsg::Loaded(Ok(reviews)) => state.planner.reviews = reviews,
        ReviewMsg::Loaded(Err(e)) => notice(&mut state, NoticeKind::Error, e),
        ReviewMsg::OpenMove(stack, day) => {
            let text = format_date(date_from_days(day));
            state.planner.moving = Some(MoveForm { stack, day, text });
        }
        ReviewMsg::MoveText(text) => {
            if let Some(form) = &mut state.planner.moving {
                form.text = text;
            }
        }
        ReviewMsg::Confirm => return confirm(state),
        ReviewMsg::Cancel => state.planner.moving = None,
        ReviewMsg::Drop { stack, from, to } => return move_to(state, stack, from, to),
        ReviewMsg::Moved(result) => return moved(state, result),
        ReviewMsg::Undo => {
            if let Some(undo) = state.planner.undo.take() {
                return (state, vec![Effect::Reviews(ReviewEffect::Undo(undo))]);
            }
        }
        ReviewMsg::Undone(Ok(())) => {
            notice(&mut state, NoticeKind::Info, "Move undone.".to_owned());
            state.library.stale = true;
            let reload = reload(&state);
            return (state, reload);
        }
        ReviewMsg::Undone(Err(e)) => notice(
            &mut state,
            NoticeKind::Error,
            format!("Could not undo: {e}"),
        ),
        ReviewMsg::OpenStack(stack) => return open_stack(state, stack),
        ReviewMsg::Show(on) => return show(state, on),
    }
    (state, Vec::new())
}

fn show(state: State, on: bool) -> (State, Vec<Effect>) {
    let app = studyfrog_core::settings::AppSettings {
        calendar_reviews_off: !on,
        ..state.app.clone()
    };
    let mut effects = vec![Effect::SaveAppSettings(app.clone())];
    let mut state = State { app, ..state };
    if on {
        effects.extend(reload(&state));
    } else {
        state.planner.reviews.clear();
    }
    (state, effects)
}
