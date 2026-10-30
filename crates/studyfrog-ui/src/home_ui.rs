//! The home screen's update logic and the shortcuts that start a rehearsal from it. Pure.

use studyfrog_service::home::HomeSummary;

use crate::effect::Effect;
use crate::msg::Msg;
use crate::nav::navigate;
use crate::run_update;
use crate::setup::{Preset, Rehearse, SetupEdit, apply_edit, form_for_preset, refresh_preview};
use crate::state::{Notice, NoticeKind, Screen, State};

/// Messages of the home screen.
#[derive(Debug, Clone, PartialEq)]
pub enum HomeMsg {
    /// The summary was read.
    Loaded(Result<Box<HomeSummary>, String>),
    /// Start today's spaced run right away.
    StartToday,
    /// Open the run setup for one stack (and the stacks inside it).
    RehearseStack(String),
}

/// Keep the summary; a failure is reported and the last numbers stay.
fn loaded(state: State, result: Result<Box<HomeSummary>, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(h) => (
            State {
                home: Some(*h),
                ..state
            },
            Vec::new(),
        ),
        Err(e) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not read the home screen: {e}"),
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}

fn start_today(state: State) -> (State, Vec<Effect>) {
    let form = form_for_preset(Preset::Spaced, state.rehearse.form.mode);
    let state = State {
        rehearse: Rehearse {
            form,
            run: None,
            ..state.rehearse.clone()
        },
        screen: Screen::Rehearse,
        ..state
    };
    run_update::update(state, Msg::StartRunClicked)
}

fn rehearse_stack(state: State, folder: &str) -> (State, Vec<Effect>) {
    let uids: Vec<_> = state
        .vault
        .iter()
        .flat_map(|v| v.overview.stacks.iter())
        .filter(|s| s.folder == folder || s.folder.starts_with(&format!("{folder}/")))
        .map(|s| s.uid)
        .collect();
    let form = apply_edit(
        form_for_preset(Preset::Spaced, state.rehearse.form.mode),
        SetupEdit::Stacks(uids, true),
    );
    let (state, _) = navigate(
        State {
            rehearse: Rehearse {
                form,
                run: None,
                ..state.rehearse.clone()
            },
            ..state
        },
        Screen::Rehearse,
    );
    refresh_preview(state)
}

/// Handle the home screen's messages.
pub fn update(state: State, msg: HomeMsg) -> (State, Vec<Effect>) {
    match msg {
        HomeMsg::Loaded(result) => loaded(state, result),
        HomeMsg::StartToday => start_today(state),
        HomeMsg::RehearseStack(folder) => rehearse_stack(state, &folder),
    }
}
