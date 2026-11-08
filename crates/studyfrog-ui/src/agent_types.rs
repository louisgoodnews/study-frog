//! The state and messages of the assistant screen (ADR 0009).

use studyfrog_agent::exam::{ExamItem, ExamResult};
use studyfrog_agent::provider::{Message, Protocol};
use studyfrog_agent::wiki::{ContextItem, Proposal};
use studyfrog_service::agent::Connections;
use studyfrog_service::agent_vault::{Applied, WikiContext};

use crate::state::VaultInfo;

/// The three parts of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentTab {
    /// The models the user connected.
    #[default]
    Connections,
    /// Working the vault like a wiki.
    Wiki,
    /// Being examined.
    Exam,
    /// Ready-made prompts for an AI.
    Prompts,
}

/// The form for a new connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnForm {
    /// Name.
    pub name: String,
    /// Protocol.
    pub protocol: Protocol,
    /// Address.
    pub base_url: String,
    /// Model.
    pub model: String,
    /// API key (kept only until it is saved).
    pub key: String,
}

/// A request waiting for the user's go-ahead: what will be sent where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    /// The stacks.
    pub stacks: Vec<(String, String)>,
    /// The items that would be sent.
    pub items: Vec<ContextItem>,
    /// Characters of item text.
    pub chars: usize,
    /// Items left out for lack of room.
    pub left_out: usize,
}

/// A proposal with the user's choice for each change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reviewing {
    /// What the model proposed.
    pub proposal: Proposal,
    /// Whether each change is accepted.
    pub accepted: Vec<bool>,
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

/// Everything about the assistant screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentState {
    /// The part on screen.
    pub tab: AgentTab,
    /// Connections and which have keys.
    pub connections: Connections,
    /// The connection in use.
    pub selected: Option<String>,
    /// The stacks the agent may read (folders).
    pub folders: Vec<String>,
    /// The new-connection form.
    pub form: ConnForm,
    /// What to do with the vault.
    pub instruction: String,
    /// Pages of a PDF to read as well: path, first and last page (0-based).
    pub pdf_pages: Option<(String, usize, usize)>,
    /// Read the user's own help pages as well.
    pub help_pages: bool,
    /// A wiki request waiting for the go-ahead.
    pub pending: Option<Pending>,
    /// A proposal being reviewed.
    pub reviewing: Option<Reviewing>,
    /// What applying a proposal did.
    pub applied: Option<Applied>,
    /// The exam.
    pub exam: ExamState,
    /// How many questions the exam asks.
    pub exam_count: String,
    /// A request is running.
    pub busy: bool,
    /// The last thing that went wrong.
    pub error: Option<String>,
    /// What AI clients staged or changed over MCP.
    pub mcp: crate::agent_mcp_types::McpUi,
}

impl Default for AgentState {
    fn default() -> Self {
        AgentState {
            tab: AgentTab::default(),
            connections: Connections::default(),
            selected: None,
            folders: Vec::new(),
            form: ConnForm {
                name: String::new(),
                protocol: Protocol::Ollama,
                base_url: studyfrog_agent::provider::default_base_url(Protocol::Ollama).to_owned(),
                model: String::new(),
                key: String::new(),
            },
            instruction: "Organise these notes: add an Index note, link related notes, and make flashcards of the key facts.".to_owned(),
            pdf_pages: None,
            help_pages: false,
            pending: None,
            reviewing: None,
            applied: None,
            exam: ExamState::default(),
            exam_count: "5".to_owned(),
            busy: false,
            error: None,
            mcp: crate::agent_mcp_types::McpUi::default(),
        }
    }
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

/// Messages of the assistant screen.
#[derive(Debug, Clone, PartialEq)]
pub enum AgentMsg {
    /// The connections were read.
    Loaded(Connections),
    /// Another part of the screen.
    Tab(AgentTab),
    /// Use this connection.
    Select(String),
    /// A stack may be read or not.
    Folder(String, bool),
    /// A field of the new-connection form.
    Form(FormEdit),
    /// Save the new connection.
    AddConnection,
    /// Remove a connection and its key.
    RemoveConnection(String),
    /// Allow or forbid sending vault text to a remote connection.
    Allow(String, bool),
    /// The connections changed (saved or removed).
    Saved(Result<Connections, String>),
    /// The instruction for the wiki changed.
    Instruction(String),
    /// Read these pages of a PDF as well (path, first, last; 0-based).
    UsePdfPages(String, usize, usize),
    /// Do not read PDF pages.
    ClearPdfPages,
    /// Read the user's help pages as well, or not.
    UseHelpPages(bool),
    /// Look at what a wiki request would send.
    PrepareWiki,
    /// That is ready: what would be sent.
    WikiPrepared(Result<Box<WikiContext>, String>),
    /// Send it.
    SendWiki,
    /// The model answered with a proposal.
    WikiReplied(Result<Box<Proposal>, String>),
    /// Accept or reject one change.
    Accept(usize, bool),
    /// Apply the accepted changes.
    ApplyProposal,
    /// The changes were applied; carries the refreshed vault.
    ProposalApplied(Result<Box<(Applied, VaultInfo)>, String>),
    /// Drop the pending request or the proposal.
    Discard,
    /// How many exam questions changed.
    ExamCount(String),
    /// Start an exam.
    StartExam,
    /// The questions were chosen.
    ExamPrepared(Result<Vec<ExamItem>, String>),
    /// What the student types changed.
    ExamInput(String),
    /// Send the student's answer.
    ExamSend,
    /// The examiner answered.
    ExamReplied(Result<String, String>),
    /// Record the verdicts as reviews.
    RecordExam,
    /// The verdicts were recorded.
    ExamRecorded(Result<u32, String>),
    /// Staged proposals and undo entries of AI clients.
    Mcp(crate::agent_mcp_types::McpMsg),
}

pub use crate::agent_conn::{accepted_changes, selected_provider};

/// What a model answer is for, so the worker knows how to read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatPurpose {
    /// A wiki proposal; carries the item ids and stack folders the model was shown.
    Wiki(Vec<String>, Vec<String>),
    /// An exam turn.
    Exam,
    /// A standard prompt run with an attached file.
    Prompt,
}
