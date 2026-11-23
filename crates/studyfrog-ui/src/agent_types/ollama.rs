//! The local-model setup on the assistant's Connections tab (STORY-0206): the audit, the Ollama
//! status, the suggestions and the guide. Probes run as effects. A model is tested with one short
//! prompt before it is saved as the local connection.

use studyfrog_service::agent::Connections;
use studyfrog_service::ollama::audit_types::Audit;
use studyfrog_service::ollama::client::OllamaStatus;
use studyfrog_service::ollama::fit::Suggestion;
use studyfrog_service::ollama::guide::Step;

use crate::agent_mcp_types::AgentEffect;
use crate::agent_types::AgentMsg;
use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::State;

/// The name of the connection the setup saves the local model under.
pub const LOCAL_NAME: &str = "Local model (Ollama)";

/// What one check found: the audit, the service, the ranked suggestions and the guide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// The hardware audit.
    pub audit: Audit,
    /// The Ollama service.
    pub status: OllamaStatus,
    /// The catalog ranked for this computer.
    pub suggestions: Vec<Suggestion>,
    /// The setup steps from the state found.
    pub guide: Vec<Step>,
    /// Why the catalog could not be read, if it could not.
    pub catalog_error: Option<String>,
}

/// Messages of the setup panel.
#[derive(Debug, Clone, PartialEq)]
pub enum OllamaMsg {
    /// Check this computer and the Ollama service again.
    Refresh,
    /// A check finished.
    Loaded(Box<Snapshot>),
    /// Test this model with one short prompt, then use it.
    Choose(String),
    /// The test ended: the model's name and what came back.
    Tested(String, Result<String, String>),
    /// The local connection was saved (or why not).
    LocalSet(Result<Connections, String>),
}

/// The panel's state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OllamaPanel {
    /// The last check, if any.
    pub snapshot: Option<Box<Snapshot>>,
    /// A check is running.
    pub busy: bool,
    /// The model being tested now.
    pub testing: Option<String>,
    /// The last test: the model and its outcome.
    pub test: Option<(String, Result<String, String>)>,
    /// Why the local model could not be saved.
    pub error: Option<String>,
}

/// Handle a message of the panel; the state is changed in place.
pub fn update(mut state: State, msg: OllamaMsg) -> (State, Vec<Effect>) {
    let effects = match msg {
        OllamaMsg::Refresh => {
            state.agent.ollama.busy = true;
            vec![AgentEffect::RefreshOllama]
        }
        OllamaMsg::Loaded(snap) => {
            state.agent.ollama.busy = false;
            state.agent.ollama.snapshot = Some(snap);
            Vec::new()
        }
        OllamaMsg::Choose(name) => choose(&mut state, name),
        OllamaMsg::Tested(name, result) => tested(&mut state, name, result),
        OllamaMsg::LocalSet(result) => {
            local_set(&mut state, result);
            Vec::new()
        }
    };
    (state, effects.into_iter().map(Effect::Agent).collect())
}

fn choose(state: &mut State, name: String) -> Vec<AgentEffect> {
    let p = &mut state.agent.ollama;
    if p.testing.is_some() || name.trim().is_empty() {
        return Vec::new();
    }
    p.testing = Some(name.clone());
    p.test = None;
    p.error = None;
    vec![AgentEffect::TestLocalModel(name)]
}

fn tested(state: &mut State, name: String, result: Result<String, String>) -> Vec<AgentEffect> {
    let ok = result.is_ok();
    let p = &mut state.agent.ollama;
    p.testing = None;
    p.test = Some((name.clone(), result));
    if ok {
        vec![AgentEffect::SetLocalModel(name)]
    } else {
        Vec::new()
    }
}

fn local_set(state: &mut State, result: Result<Connections, String>) {
    match result {
        Ok(c) => {
            state.agent.connections = c;
            state.agent.selected = Some(LOCAL_NAME.to_owned());
            state.agent.ollama.error = None;
        }
        Err(e) => state.agent.ollama.error = Some(e),
    }
}

/// The message as a whole-screen message (for views and the control protocol).
pub fn msg(m: OllamaMsg) -> Msg {
    Msg::Agent(AgentMsg::Ollama(m))
}
