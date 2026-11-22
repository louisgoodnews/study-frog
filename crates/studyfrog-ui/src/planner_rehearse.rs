//! Starting a rehearsal from a calendar entry.

use studyfrog_core::uid::Uid;

use crate::effect::Effect;
use crate::setup::memory::MemoryMsg;
use crate::setup::{Preset, Rehearse, SetupEdit, apply_edit, form_for_preset, refresh_preview};
use crate::state::{Screen, State};

/// Open the rehearse screen with the session's stack or the stack directly as the filter.
pub fn rehearse(state: State, id: Uid) -> (State, Vec<Effect>) {
    let session = state.planner.sessions.iter().find(|s| s.id == id);
    let stack_uid = session.and_then(|s| s.stack).or_else(|| {
        state.vault.as_ref().and_then(|v| {
            v.overview
                .stacks
                .iter()
                .find(|s| s.uid == id)
                .map(|s| s.uid)
        })
    });
    let mut form = form_for_preset(Preset::Spaced, state.rehearse.form.mode);
    if let Some(uid) = stack_uid {
        let uids: Vec<Uid> = state
            .vault
            .as_ref()
            .and_then(|v| {
                let target = v.overview.stacks.iter().find(|s| s.uid == uid)?;
                let folder = &target.folder;
                Some(
                    v.overview
                        .stacks
                        .iter()
                        .filter(|s| {
                            s.folder == *folder || s.folder.starts_with(&format!("{folder}/"))
                        })
                        .map(|s| s.uid)
                        .collect(),
                )
            })
            .unwrap_or_else(|| vec![uid]);
        form = apply_edit(form, SetupEdit::Stacks(uids, true));
    }
    refresh_preview(State {
        screen: Screen::Rehearse,
        rehearse: Rehearse {
            form,
            run: None,
            memory: None,
            ..state.rehearse.clone()
        },
        ..state
    })
}

/// Open the rehearse screen and start a memory game with the cards of `stack`, as the stack page does.
pub fn memory(state: State, stack: Uid) -> (State, Vec<Effect>) {
    let folder = state.vault.as_ref().and_then(|v| {
        v.overview
            .stacks
            .iter()
            .find(|s| s.uid == stack)
            .map(|s| s.folder.clone())
    });
    let Some(folder) = folder else {
        return (state, Vec::new());
    };
    let (state, mut effects) = crate::nav::navigate(state, Screen::Rehearse);
    if state.screen != Screen::Rehearse {
        return (state, effects);
    }
    let (state, more) = crate::setup::memory::update(state, MemoryMsg::StartStack(folder));
    effects.extend(more);
    (state, effects)
}
