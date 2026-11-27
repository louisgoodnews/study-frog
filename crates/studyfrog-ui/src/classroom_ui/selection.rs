//! Choices the Classroom screen reads from the state and the session: the model, the simulated
//! students, the user's seat and the source checkboxes. Re-exported from `classroom_ui`.

use studyfrog_agent::provider::Provider;
use studyfrog_core::classroom::{ClassroomSession, Participant, SourceKind, SourceRef};
use studyfrog_core::uid::Uid;

use crate::state::State;

/// The model the screen uses: the one chosen here, else the Assistant's selection, else the first.
pub fn chosen_provider(state: &State) -> Result<Provider, String> {
    let providers = &state.agent.connections.providers;
    let name = state
        .classroom
        .provider
        .as_ref()
        .or(state.agent.selected.as_ref());
    let found = match name {
        Some(name) => providers.iter().find(|p| &p.name == name),
        None => providers.first(),
    };
    found
        .or_else(|| providers.first())
        .cloned()
        .ok_or_else(|| "Connect a model in Assistant first.".to_owned())
}

pub(super) fn check_student(session: &ClassroomSession, index: usize) -> Result<(), String> {
    match session.participants.get(index) {
        Some(Participant::SimulatedStudent { .. }) => Ok(()),
        _ => Err("Only a simulated student can ask a question.".to_owned()),
    }
}

/// The participant index of the user, if the session has one.
pub fn user_index(session: &ClassroomSession) -> Option<usize> {
    session
        .participants
        .iter()
        .position(|p| matches!(p, Participant::User { .. }))
}

/// Whether a source of this kind is in the list, for the form's checkboxes.
pub fn is_chosen(sources: &[SourceRef], kind: SourceKind, uid: Uid) -> bool {
    sources.iter().any(|s| s.kind == kind && s.uid == uid)
}
