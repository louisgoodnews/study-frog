//! Starting a rehearsal from a calendar entry.

use studyfrog_core::uid::Uid;

use crate::effect::Effect;
use crate::setup::{Preset, Rehearse, SetupEdit, apply_edit, form_for_preset, refresh_preview};
use crate::state::{Screen, State};

/// Open the rehearse screen with the session's stack as the only stack filter.
pub fn rehearse(state: State, id: Uid) -> (State, Vec<Effect>) {
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
