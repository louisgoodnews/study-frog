//! The saved-session browser's reducer (TASK-0295): resume, Markdown export and confirmed delete.
//! Each action asks the worker; the list is read back after every change.

use crate::classroom_types::{ClassroomEffect, SavedEffect, SavedMsg};
use crate::effect::Effect;
use crate::state::State;

/// Asks the worker to read the saved sessions back.
pub fn refresh() -> Vec<Effect> {
    vec![saved(SavedEffect::List)]
}

/// Advance the browser by one message and return the effects to run.
pub fn apply(state: &mut State, msg: SavedMsg) -> Vec<Effect> {
    match msg {
        SavedMsg::Resume(id) => resume(state, id),
        SavedMsg::Export(id) => vec![saved(SavedEffect::Export { id })],
        SavedMsg::AskDelete(id) => {
            state.classroom.confirm_delete = Some(id);
            Vec::new()
        }
        SavedMsg::CancelDelete => {
            state.classroom.confirm_delete = None;
            Vec::new()
        }
        SavedMsg::ConfirmDelete => match state.classroom.confirm_delete.take() {
            Some(id) => vec![saved(SavedEffect::Delete { id })],
            None => Vec::new(),
        },
        SavedMsg::Listed {
            sessions,
            notice,
            error,
        } => {
            state.classroom.saved = sessions;
            state.classroom.saved_notice = notice;
            if error.is_some() {
                state.classroom.error = error;
            }
            Vec::new()
        }
    }
}

/// Loads the saved session onto the screen. Like a turn, the load holds the controls until the
/// worker answers, but reading the session back needs no model.
fn resume(state: &mut State, id: studyfrog_core::uid::Uid) -> Vec<Effect> {
    if state.classroom.busy {
        return Vec::new();
    }
    state.classroom.open_task = None;
    state.classroom.busy = true;
    state.classroom.error = None;
    vec![saved(SavedEffect::Load { id })]
}

fn saved(effect: SavedEffect) -> Effect {
    Effect::Classroom(ClassroomEffect::Saved(effect))
}
