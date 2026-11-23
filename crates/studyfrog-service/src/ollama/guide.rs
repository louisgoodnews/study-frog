//! The setup guide: four steps (install Ollama, start the service, pull a model, run it), the same
//! for every system, with the command for each system in a copyable block. The step after the last
//! finished one is the next step. Pure.

use super::audit_types::Os;
use super::client::{OllamaState, OllamaStatus};

/// One step of the guide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// The step's title.
    pub title: String,
    /// The commands to copy (empty when the step is done in a window).
    pub commands: Vec<String>,
    /// What to know about the step.
    pub note: String,
    /// The state found already covers this step.
    pub done: bool,
    /// The first step not done: highlighted.
    pub next: bool,
}

/// The guide for this state. `model` is the tag to pull and run: the best suggestion's name.
pub fn steps(os: Os, status: &OllamaStatus, model: &str) -> Vec<Step> {
    let state = status.state;
    let installed = state != OllamaState::Missing;
    let serving = matches!(state, OllamaState::NoModels | OllamaState::Ready);
    let pulled = state == OllamaState::Ready;
    let mut out = vec![
        step(
            "Install Ollama",
            install_commands(os),
            "Ollama runs the models on this computer. Nothing is installed until you run the command.",
            installed,
        ),
        step(
            "Start the Ollama service",
            vec!["ollama serve".to_owned()],
            "Leave it running. On Windows and macOS the Ollama app starts the service for you.",
            serving,
        ),
        step(
            "Pull a model",
            vec![format!("ollama pull {model}")],
            "Downloads the model to this computer; it can be large. Check the download size first.",
            pulled,
        ),
        step(
            "Run the model",
            vec![format!("ollama run {model}")],
            "Type a question to try it, then /bye to leave. StudyFrog checks the model with a test prompt too.",
            false,
        ),
    ];
    if let Some(first) = out.iter().position(|s| !s.done) {
        out[first].next = true;
    }
    out
}

fn step(title: &str, commands: Vec<String>, note: &str, done: bool) -> Step {
    Step {
        title: title.to_owned(),
        commands,
        note: note.to_owned(),
        done,
        next: false,
    }
}

/// The install command for each system; a system without one gets a note in the step instead.
pub fn install_commands(os: Os) -> Vec<String> {
    let cmd = match os {
        Os::Linux => "curl -fsSL https://ollama.com/install.sh | sh",
        Os::MacOs => "brew install ollama",
        Os::Windows => "winget install --id Ollama.Ollama",
        Os::Other => return Vec::new(),
    };
    vec![cmd.to_owned()]
}
