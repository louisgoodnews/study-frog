//! The page of a stack: its tabs (the last one is remembered per stack), the Summary read when its tab opens, and the status
//! chip of the header. Pure.

use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;
use studyfrog_service::stack_summary::StackSummary;

use crate::effect::Effect;
use crate::library_bulk::LibraryEffect;
pub use crate::stack_goals::{Goals, sync_goals};
use crate::stack_tabs::remembered;
pub use crate::stack_tabs::{StackTab, TABS, tab_label, tab_of};
use crate::state::{Notice, NoticeKind, Screen, Selection, State, VaultInfo};

/// The summary read for one stack, and whether one is being read.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StackViewState {
    /// The folder, the library generation it was read at, and the summary.
    pub summary: Option<(String, u64, Box<StackSummary>)>,
    /// The folder and generation being read.
    pub loading: Option<(String, u64)>,
    /// The goals of the open stack: its folder, what is saved and what is typed.
    pub goals: Option<Goals>,
    /// How the roadmap is zoomed.
    pub zoom: crate::stack_roadmap::Zoom,
}

/// What happened on the stack page.
#[derive(Debug, Clone, PartialEq)]
pub enum StackViewMsg {
    /// Show a tab of the open stack.
    Select(StackTab),
    /// The next (`1`) or previous (`-1`) tab.
    Step(i32),
    /// The summary of a stack arrived.
    Loaded(String, u64, Result<Box<StackSummary>, String>),
    /// The goals of a stack were read.
    GoalsLoaded(String, Result<String, String>),
    /// The goals text was edited.
    GoalsEdited(String),
    /// Write the goals to the stack's file.
    GoalsSave,
    /// The goals were written.
    GoalsSaved(Result<Box<VaultInfo>, String>),
    /// Zoom the roadmap.
    Zoom(crate::stack_roadmap::Zoom),
    /// Give the stack a status (`None`: automatic).
    SetStatus(Uid, Option<Status>),
    /// The status was written.
    StatusSet(Result<Box<VaultInfo>, String>),
}

pub(crate) fn open_folder(state: &State) -> Option<&str> {
    match &state.browser.selection {
        Selection::Stack(f) if state.screen == Screen::Library => Some(f),
        _ => None,
    }
}

fn select(state: State, tab: StackTab) -> (State, Vec<Effect>) {
    let Some(folder) = open_folder(&state).map(str::to_owned) else {
        return (state, Vec::new());
    };
    let app = remembered(&state.app, &folder, tab);
    let effects = vec![Effect::SaveAppSettings(app.clone())];
    (State { app, ..state }, effects)
}

fn notice(state: &mut State, kind: NoticeKind, text: String) {
    state.notice = Some(Notice { kind, text });
}

/// Handle a message of the stack page.
pub fn update(state: State, msg: StackViewMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    match msg {
        StackViewMsg::Select(tab) => return select(state, tab),
        StackViewMsg::Step(by) => {
            let at = open_folder(&state).map_or(0, |f| {
                TABS.iter()
                    .position(|(t, _, _)| *t == tab_of(&state.app, f))
                    .unwrap_or(0)
            });
            let next = (at as i32 + by).rem_euclid(TABS.len() as i32) as usize;
            return select(state, TABS[next].0);
        }
        StackViewMsg::Loaded(folder, generation, Ok(summary)) => {
            state.library.stack_view.loading = None;
            state.library.stack_view.summary = Some((folder, generation, summary));
        }
        StackViewMsg::Loaded(_, _, Err(e)) => {
            state.library.stack_view.loading = None;
            notice(
                &mut state,
                NoticeKind::Error,
                format!("Could not read the summary: {e}"),
            );
        }
        StackViewMsg::GoalsLoaded(folder, Ok(text)) => {
            let goals = Goals {
                folder,
                saved: text.clone(),
                draft: text,
                saving: false,
            };
            state.library.stack_view.goals = Some(goals);
        }
        StackViewMsg::GoalsLoaded(_, Err(e)) => notice(
            &mut state,
            NoticeKind::Error,
            format!("Could not read the goals: {e}"),
        ),
        StackViewMsg::GoalsEdited(text) => {
            if let Some(g) = &mut state.library.stack_view.goals {
                g.draft = text;
            }
        }
        StackViewMsg::GoalsSave => return crate::stack_goals::save_goals(state),
        StackViewMsg::GoalsSaved(result) => return crate::stack_goals::goals_saved(state, result),
        StackViewMsg::Zoom(zoom) => state.library.stack_view.zoom = zoom,
        StackViewMsg::SetStatus(uid, status) => {
            return (
                state,
                vec![Effect::Library(LibraryEffect::SetStatus(uid, status))],
            );
        }
        StackViewMsg::StatusSet(Ok(info)) => {
            state.vault = Some(*info);
            state.library.stale = true;
            let reload = crate::browse::reload(&state.browser.selection);
            return (state, reload);
        }
        StackViewMsg::StatusSet(Err(e)) => notice(
            &mut state,
            NoticeKind::Error,
            format!("Could not set the status: {e}"),
        ),
    }
    (state, Vec::new())
}

/// Read the summary when the Summary tab of a stack is on screen and what is held is for another stack or an older list.
pub fn sync(state: State) -> (State, Vec<Effect>) {
    let mut state = state;
    let generation = state.library.generation;
    let Some(folder) = open_folder(&state).map(str::to_owned) else {
        return (state, Vec::new());
    };
    let held = state
        .library
        .stack_view
        .summary
        .as_ref()
        .is_some_and(|(f, g, _)| *f == folder && *g == generation);
    let asked = state
        .library
        .stack_view
        .loading
        .as_ref()
        .is_some_and(|(f, g)| *f == folder && *g == generation);
    let wants = matches!(
        tab_of(&state.app, &folder),
        StackTab::Summary | StackTab::Substacks
    );
    if !wants || held || asked || state.library.stale {
        return (state, Vec::new());
    }
    state.library.stack_view.loading = Some((folder.clone(), generation));
    let effect = Effect::Library(LibraryEffect::LoadSummary(folder, generation));
    (state, vec![effect])
}
