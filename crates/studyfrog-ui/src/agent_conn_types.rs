//! Connection form and exam state data structures for the assistant.

use studyfrog_agent::exam::{ExamItem, ExamResult};
use studyfrog_agent::provider::{Message, Protocol};

/// The form for a new connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnForm {
    /// Provider name.
    pub name: String,
    /// Wire protocol.
    pub protocol: Protocol,
    /// Base address.
    pub base_url: String,
    /// Model name.
    pub model: String,
    /// API key.
    pub key: String,
}

/// A field of the connection form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormEdit {
    /// The name.
    Name(String),
    /// The protocol (also resets the address to its usual one).
    Protocol(Protocol),
    /// The address.
    BaseUrl(String),
    /// The model.
    Model(String),
    /// The key.
    Key(String),
}

/// An exam in progress or finished.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExamState {
    /// The questions the examiner knows.
    pub items: Vec<ExamItem>,
    /// The conversation so far (the first turn only starts the exam).
    pub chat: Vec<Message>,
    /// What the student is typing.
    pub input: String,
    /// The verdicts, once the examiner gave them.
    pub results: Option<Vec<ExamResult>>,
    /// The verdicts were recorded as reviews.
    pub recorded: Option<u32>,
}
