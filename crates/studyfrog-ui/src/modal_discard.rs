//! Closing the Create dialog without losing what the user wrote: it asks first when something is written. Pure.

use studyfrog_core::event::Difficulty;
use studyfrog_core::priority::Priority;

use crate::effect::Effect;
use crate::modal::{CreateForm, Modal};
use crate::state::State;

/// Whether the user wrote or changed something in the form (the kind and the parent stack do not count:
/// they are choices, not content).
pub fn is_dirty(f: &CreateForm) -> bool {
    let blank = |s: &str| s.trim().is_empty();
    !(blank(&f.title) && blank(&f.text) && blank(&f.answer) && blank(&f.tags) && blank(&f.due))
        || f.priority != Priority::Medium
        || f.difficulty != Difficulty::Difficult
        || f.reminders != [1440]
}

fn ask(state: State, on: bool) -> (State, Vec<Effect>) {
    let modal = match &state.modal {
        Some(Modal::Create(f)) => Some(Modal::Create(CreateForm {
            confirm_discard: on,
            ..f.clone()
        })),
        other => other.clone(),
    };
    (State { modal, ..state }, Vec::new())
}

/// Close the dialog: at once when nothing is written, else ask; asking again (Escape, the close button)
/// withdraws the question.
pub fn close(state: State) -> (State, Vec<Effect>) {
    match &state.modal {
        Some(Modal::Create(f)) if f.confirm_discard => ask(state, false),
        Some(Modal::Create(f)) if !f.busy && is_dirty(f) => ask(state, true),
        _ => discard(state),
    }
}

/// 'Keep editing': go back to the form.
pub fn keep_editing(state: State) -> (State, Vec<Effect>) {
    ask(state, false)
}

/// 'Discard': close the dialog and lose what was written.
pub fn discard(state: State) -> (State, Vec<Effect>) {
    (
        State {
            modal: None,
            ..state
        },
        Vec::new(),
    )
}
