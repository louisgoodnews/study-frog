//! Assistant, wiki and exam effects.

/// Assistant, wiki and exam effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectAssistant {
    /// Read the assistant's connections.
    LoadAgent,
    /// Save a connection (and its key).
    SaveConnection {
        /// The connection.
        provider: studyfrog_agent::provider::Provider,
        /// Its API key, if it has one.
        key: Option<String>,
    },
    /// Collect what a wiki request would send.
    PrepareWiki {
        /// The stacks to read.
        folders: Vec<String>,
        /// Pages of a PDF to read as well: path, first and last page (0-based).
        pdf: Option<(String, usize, usize)>,
    },
    /// Ask a model one question.
    AgentChat {
        /// The model.
        provider: studyfrog_agent::provider::Provider,
        /// Its instructions.
        system: String,
        /// The conversation.
        messages: Vec<studyfrog_agent::provider::Message>,
        /// What the answer is for.
        purpose: crate::agent_types::ChatPurpose,
    },
    /// Apply the changes the user accepted.
    ApplyChanges(Vec<studyfrog_agent::wiki::Change>),
    /// Choose the questions of an exam.
    PrepareExam {
        /// The stacks.
        folders: Vec<String>,
        /// How many questions.
        count: usize,
    },
    /// Record exam verdicts as reviews.
    RecordExam(Vec<studyfrog_agent::exam::ExamResult>),
}
