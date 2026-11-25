//! The suggested Ollama model the wizard can download (STORY-0212): which suggestion is offered, and the confirmed pull
//! and its answer. Pure: the pull itself is an effect, and the model offered is read from the state, never from a message.

use studyfrog_service::ollama::client::OllamaState;
use studyfrog_service::ollama::fit::{Suggestion, fits};

use crate::agent_types::ollama::{OllamaMsg, Snapshot, update as ollama_update};
use crate::effect::Effect;
use crate::state::State;
use crate::view::wizard::types::WizardEffect;

/// The model the wizard offers: the first suggestion that fits this computer and is not installed yet. `None` when
/// the Ollama service does not answer, or when no such suggestion exists (for example because memory is unknown).
pub fn pick_of(snap: &Snapshot) -> Option<&Suggestion> {
    let answers = matches!(
        snap.status.state,
        OllamaState::Ready | OllamaState::NoModels
    );
    if !answers {
        return None;
    }
    snap.suggestions
        .iter()
        .find(|s| fits(s.fit) && !s.installed)
}

/// The user clicked the download: start the pull of the offered model, unless one runs or none is offered.
pub fn confirm_pull(mut state: State) -> (State, Vec<Effect>) {
    if !state.wizard.open || state.wizard.pulling {
        return (state, Vec::new());
    }
    let offered = state
        .agent
        .ollama
        .snapshot
        .as_deref()
        .and_then(pick_of)
        .map(|s| s.name.clone());
    let Some(name) = offered else {
        return (state, Vec::new());
    };
    state.wizard.pulling = true;
    state.wizard.pull_error = None;
    studyfrog_log::log_info!("ui.wizard", "model download confirmed by the user");
    (state, vec![Effect::Wizard(WizardEffect::PullModel(name))])
}

/// The pull ended. On success the Ollama check runs again, so the new model shows as installed.
pub fn pulled(mut state: State, result: Result<(), String>) -> (State, Vec<Effect>) {
    state.wizard.pulling = false;
    match result {
        Ok(()) => {
            studyfrog_log::log_info!(
                "ui.wizard",
                "model download finished; the Ollama service is checked again"
            );
            state.wizard.pull_error = None;
            ollama_update(state, OllamaMsg::Refresh)
        }
        Err(e) => {
            studyfrog_log::log_warn!("ui.wizard", "model download failed: {}", e);
            state.wizard.pull_error = Some(e);
            (state, Vec::new())
        }
    }
}
