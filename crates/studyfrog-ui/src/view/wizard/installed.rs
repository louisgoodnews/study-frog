//! What is already set up, read from the state (STORY-0212): the open vault, the Ollama check, the saved connections
//! and the MiniLM probe. Pure: the probe itself is an effect.

use studyfrog_config::wizard::Installed;
use studyfrog_service::ollama::client::OllamaState;

use crate::agent_types::ollama::LOCAL_NAME;
use crate::state::State;

/// The set-up parts, as the wizard's steps and summary read them.
pub fn installed_of(state: &State) -> Installed {
    Installed {
        vault: state.vault.is_some(),
        local_model: state
            .agent
            .ollama
            .snapshot
            .as_deref()
            .is_some_and(|s| matches!(s.status.state, OllamaState::Ready)),
        remote_provider: state
            .agent
            .connections
            .providers
            .iter()
            .any(|p| p.name != LOCAL_NAME),
        minilm: state.wizard.minilm_installed,
    }
}
