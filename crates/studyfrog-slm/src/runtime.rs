//! The contract every embedding runtime fulfils.

/// Why a runtime could not produce embeddings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlmError {
    /// No model is installed or it could not be loaded.
    Unavailable(String),
    /// The runtime failed while embedding.
    Failed(String),
}

impl std::fmt::Display for SlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlmError::Unavailable(why) => write!(f, "no language model is available: {why}"),
            SlmError::Failed(why) => write!(f, "the language model failed: {why}"),
        }
    }
}

impl std::error::Error for SlmError {}

/// Turns texts into vectors whose cosine similarity reflects how alike the texts are.
///
/// One vector per input text, in order. Vectors from one runtime all have the same length.
pub trait SlmRuntime {
    /// A short name for the user: how answers were compared.
    fn name(&self) -> &str;

    /// Embed the texts.
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SlmError>;
}
