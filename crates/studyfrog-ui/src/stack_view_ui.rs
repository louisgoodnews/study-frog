//! The page of a stack: its tabs (the last one is remembered per stack), the Summary read when its tab opens, and the status
//! chip of the header. Pure.

use studyfrog_core::settings::{AppSettings, MAX_STACK_TABS};
use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;
use studyfrog_service::stack_summary::StackSummary;

use crate::effect::Effect;
use crate::library_bulk::LibraryEffect;
use crate::state::{Notice, NoticeKind, Screen, Selection, State, VaultInfo};

/// A tab of the stack page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackTab {
    /// Counts, charts, activity and sub-stacks.
    Summary,
    /// The items of the stack as a table, with its due date and PDFs.
    List,
}

/// The tabs in the order shown, with their stored key and name.
pub const TABS: [(StackTab, &str, &str); 2] = [
    (StackTab::Summary, "summary", "Summary"),
    (StackTab::List, "list", "List"),
];

/// The name of a tab.
pub fn tab_label(t: StackTab) -> &'static str {
    TABS.iter()
        .find(|(x, _, _)| *x == t)
        .map_or("", |(_, _, l)| l)
}

/// The tab remembered for the stack at `folder`, else Summary.
pub fn tab_of(app: &AppSettings, folder: &str) -> StackTab {
    let key = app
        .stack_tabs
        .iter()
        .filter_map(|e| e.rsplit_once(':'))
        .find(|(f, _)| *f == folder)
        .map(|(_, k)| k);
    TABS.iter()
        .find(|(_, k, _)| Some(*k) == key)
        .map_or(StackTab::Summary, |(t, _, _)| *t)
}

/// The summary read for one stack, and whether one is being read.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StackViewState {
    /// The folder, the library generation it was read at, and the summary.
    pub summary: Option<(String, u64, Box<StackSummary>)>,
    /// The folder and generation being read.
    pub loading: Option<(String, u64)>,
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
    /// Give the stack a status (`None`: automatic).
    SetStatus(Uid, Option<Status>),
    /// The status was written.
    StatusSet(Result<Box<VaultInfo>, String>),
}

fn open_folder(state: &State) -> Option<&str> {
    match &state.browser.selection {
        Selection::Stack(f) if state.screen == Screen::Library => Some(f),
        _ => None,
    }
}

fn remembered(app: &AppSettings, folder: &str, tab: StackTab) -> AppSettings {
    let key = TABS
        .iter()
        .find(|(t, _, _)| *t == tab)
        .map_or("summary", |(_, k, _)| *k);
    let mut tabs: Vec<String> = app
        .stack_tabs
        .iter()
        .filter(|e| e.rsplit_once(':').is_none_or(|(f, _)| f != folder))
        .cloned()
        .collect();
    tabs.push(format!("{folder}:{key}"));
    let skip = tabs.len().saturating_sub(MAX_STACK_TABS);
    AppSettings {
        stack_tabs: tabs.split_off(skip),
        ..app.clone()
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
    if tab_of(&state.app, &folder) != StackTab::Summary || held || asked || state.library.stale {
        return (state, Vec::new());
    }
    state.library.stack_view.loading = Some((folder.clone(), generation));
    let effect = Effect::Library(LibraryEffect::LoadSummary(folder, generation));
    (state, vec![effect])
}
