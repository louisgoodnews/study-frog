//! Making a card or a question from a passage of a PDF.

use studyfrog_service::session::VaultInfo;

/// What is being made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftKind {
    /// A flashcard.
    Card,
    /// A question with a reference answer.
    Question,
}

/// The card or question being written.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// Card or question.
    pub kind: DraftKind,
    /// The front or the prompt.
    pub front: String,
    /// The back or the reference answer.
    pub back: String,
    /// The page it is made from (0-based).
    pub page: usize,
    /// Something is being made.
    pub busy: bool,
}

/// The request sent to the worker.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemJob {
    /// Vault-relative path of the PDF.
    pub path: String,
    /// Card or question.
    pub kind: DraftKind,
    /// The page (0-based).
    pub page: usize,
    /// Front or prompt.
    pub front: String,
    /// Back or reference answer.
    pub back: String,
    /// The stack folder.
    pub folder: String,
}

/// The vault as it is now and the new item's path, or why not.
pub type ItemMade = Result<Box<(VaultInfo, String)>, String>;

/// Changes to the draft.
#[derive(Debug, Clone, PartialEq)]
pub enum DraftMsg {
    /// Start a draft from a mark (by id) or from the selected words (`None`).
    Start(DraftKind, Option<u32>),
    /// The front changed.
    Front(String),
    /// The back changed.
    Back(String),
    /// Make it in the chosen stack.
    Create,
    /// Throw the draft away.
    Cancel,
    /// It was made, or not.
    Made(ItemMade),
}
