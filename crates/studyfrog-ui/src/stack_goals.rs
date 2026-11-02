//! The goals being edited on the Goals tab of a stack. Pure.

use crate::effect::Effect;
use crate::library_bulk::LibraryEffect;
use crate::stack_tabs::{StackTab, tab_of};
use crate::stack_view_ui::open_folder;
use crate::state::{Notice, NoticeKind, State, VaultInfo};

fn notice(state: &mut State, kind: NoticeKind, text: String) {
    state.notice = Some(Notice { kind, text });
}

/// The goals being edited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goals {
    /// The stack's folder.
    pub folder: String,
    /// What the file holds.
    pub saved: String,
    /// What is typed.
    pub draft: String,
    /// A save is under way.
    pub saving: bool,
}

pub(crate) fn save_goals(state: State) -> (State, Vec<Effect>) {
    let mut state = state;
    let uid = open_folder(&state).and_then(|f| {
        let stacks = &state.vault.as_ref()?.overview.stacks;
        stacks.iter().find(|s| s.folder == f).map(|s| s.uid)
    });
    let (Some(uid), Some(g)) = (uid, state.library.stack_view.goals.as_mut()) else {
        return (state, Vec::new());
    };
    g.saving = true;
    let effect = Effect::Library(LibraryEffect::SaveGoals(uid, g.draft.clone()));
    (state, vec![effect])
}

pub(crate) fn goals_saved(
    state: State,
    result: Result<Box<VaultInfo>, String>,
) -> (State, Vec<Effect>) {
    let mut state = state;
    if let Some(g) = &mut state.library.stack_view.goals {
        g.saving = false;
    }
    match result {
        Ok(info) => {
            state.vault = Some(*info);
            if let Some(g) = &mut state.library.stack_view.goals {
                g.saved = g.draft.clone();
            }
            notice(&mut state, NoticeKind::Info, "Goals saved.".to_owned());
        }
        Err(e) => notice(
            &mut state,
            NoticeKind::Error,
            format!("Could not save the goals: {e}"),
        ),
    }
    (state, Vec::new())
}

/// Read the goals when the Goals tab of a stack is on screen and the ones held are for another stack.
pub fn sync_goals(state: State) -> (State, Vec<Effect>) {
    let Some(folder) = open_folder(&state).map(str::to_owned) else {
        return (state, Vec::new());
    };
    let held = state
        .library
        .stack_view
        .goals
        .as_ref()
        .is_some_and(|g| g.folder == folder);
    if tab_of(&state.app, &folder) != StackTab::Goals || held {
        return (state, Vec::new());
    }
    let mut state = state;
    state.library.stack_view.goals = Some(Goals {
        folder: folder.clone(),
        saved: String::new(),
        draft: String::new(),
        saving: true,
    });
    let uid = state.vault.as_ref().and_then(|v| {
        v.overview
            .stacks
            .iter()
            .find(|s| s.folder == folder)
            .map(|s| s.uid)
    });
    let effect = uid.map(|u| Effect::Library(LibraryEffect::LoadGoals(folder, u)));
    (state, effect.into_iter().collect())
}
