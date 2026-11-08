//! The changes the assistant or an AI client may propose (ADR 0009): nothing is written until the user
//! applies them, or, for an MCP client, until the server was allowed to write.

/// A change the model proposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// A new note.
    CreateNote {
        /// The stack folder.
        stack: String,
        /// Title.
        title: String,
        /// Markdown body.
        body: String,
    },
    /// A new card.
    CreateCard {
        /// The stack folder.
        stack: String,
        /// Front.
        front: String,
        /// Back.
        back: String,
        /// The id of the PDF page it comes from, when the model was given PDF pages.
        source: Option<String>,
    },
    /// A new question.
    CreateQuestion {
        /// The stack folder.
        stack: String,
        /// The question.
        prompt: String,
        /// The reference answer.
        reference: String,
        /// The id of the PDF page it comes from, when the model was given PDF pages.
        source: Option<String>,
    },
    /// A new help page of the user, in the vault's `wiki/` folder.
    CreatePage {
        /// The file name without `.md`.
        name: String,
        /// The Markdown.
        text: String,
    },
    /// New text for one of the user's help pages (never a supplied page).
    UpdatePage {
        /// The file name without `.md`.
        name: String,
        /// The Markdown.
        text: String,
    },
    /// New text for parts of an item (`body`, `front`, `back`, `prompt` or `reference`).
    UpdateItem {
        /// The item's id as text.
        uid: String,
        /// The parts and their new text.
        fields: Vec<(String, String)>,
    },
    /// A learning diary entry (from an MCP client).
    LogDiary(DiaryChange),
    /// A planned study session on the calendar (from an MCP client).
    PlanSession(SessionChange),
}

/// A diary entry to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiaryChange {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// A title of its own, or empty for the default.
    pub title: String,
    /// The reflection, Markdown.
    pub body: String,
    /// Minutes studied, if given.
    pub minutes: Option<u32>,
    /// Self-assessment ratings: scale and 1 to 5.
    pub ratings: Vec<(String, u8)>,
    /// Stack folders the entry is about.
    pub stacks: Vec<String>,
}

/// A calendar entry to plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionChange {
    /// Its title.
    pub title: String,
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// Start, minutes after local midnight.
    pub start_minute: u16,
    /// Length in minutes.
    pub duration_min: u16,
    /// The stack folder to rehearse, if any.
    pub stack: Option<String>,
    /// `study`, `exam`, `task` or `appointment`.
    pub kind: String,
    /// Notes.
    pub notes: String,
}
