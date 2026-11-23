//! The local-model setup commands of the control protocol (STORY-0206): `ollama status` and
//! `ollama suggest`. Both check this computer and the local Ollama service; neither changes anything.

/// What `ollama ...` asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OllamaCmd {
    /// The service's state, version and installed models.
    Status,
    /// The models suggested for this computer, ranked by fit.
    Suggest,
}
