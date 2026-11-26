//! The data of the dialogs and toasts, and the small pure functions about it.

use studyfrog_service::create_more::{Created, NewKind};

use crate::state::VaultInfo;

/// Which modal is open.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    /// Make a stack, note, card or question.
    Create(CreateForm),
    /// Choose the folder to use as the vault.
    Vault,
    /// Add cards from a card type or a file into the open stack.
    Import,
    /// Export part of the library or import a file with a preview.
    Transfer,
    /// Confirm turning a stack into a normal folder.
    ConfirmUnmark(Box<studyfrog_service::stack_mark::UnmarkPlan>),
    /// The victory summary of a memory game.
    MemorySummary(
        Box<studyfrog_rehearsal::memory_summary::VictorySummary>,
        Option<u64>,
    ),
    /// Export items or stacks to PDF.
    PdfExport(Box<crate::pdf_export_types::PdfExportState>),
    /// Edit an item during rehearsal.
    EditItem(Box<crate::rehearsal_edit::EditCardModal>),
    /// Confirm deleting an item from the editor.
    DeleteItem(studyfrog_core::uid::Uid, String),
    /// The learn-mode editor (STORY-0198).
    LearnEditor(Box<crate::learn_editor::LearnEditor>),
    /// The scheduled and event-bound jobs (STORY-0151).
    ScriptJobs(Box<crate::console_ui::jobs::JobsEditor>),
    /// Configure mock exam item formats before generating (STORY-0207).
    MockExamFormat(Box<crate::setup::mock_exam::format_modal::MockExamFormatModal>),
}

/// The priority and difficulty the user chose; `None` leaves the default.
pub type Choices = (
    Option<studyfrog_core::priority::Priority>,
    Option<studyfrog_core::event::Difficulty>,
);

/// The create dialog's fields.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateForm {
    /// What is being created.
    pub kind: NewKind,
    /// Title.
    pub title: String,
    /// Description, body, front or prompt, depending on the kind.
    pub text: String,
    /// Back or reference answer.
    pub answer: String,
    /// Tags, comma separated.
    pub tags: String,
    /// The stack it goes into (vault-relative folder).
    pub parent: Option<String>,
    /// Another stack that references the item (vault-relative folder, optional).
    pub also_in: Option<String>,
    /// Keep the dialog open for the next one.
    pub another: bool,
    /// Why the last attempt was refused.
    pub error: Option<String>,
    /// Waiting for the vault to write it.
    pub busy: bool,
    /// For a stack: the date it is due, `YYYY-MM-DD`, or empty.
    pub due: String,
    /// For a stack with a due date: minutes before it at which to remind.
    pub reminders: Vec<u32>,
    /// The priority of a card, note or question (medium unless changed).
    pub priority: studyfrog_core::priority::Priority,
    /// The difficulty of a card, note or question (the highest unless changed).
    pub difficulty: studyfrog_core::event::Difficulty,
    /// Closing was asked for while something is written: the question 'Discard what you wrote?' is showing.
    pub confirm_discard: bool,
    /// When creating a stack/folder: whether to mark it as a stack (`_stack.md`).
    pub is_stack: bool,
    /// For a question: open text, single choice or multiple select (EPIC-072).
    pub answer_type: crate::view::choice_body::AnswerType,
    /// Active complexity warning for this WIP draft (EPIC-077, STORY-0252).
    pub complexity_warning: Option<String>,
    /// Whether the user dismissed the complexity warning for the current draft.
    pub complexity_dismissed: bool,
}

/// A field of the create dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// The title.
    Title,
    /// The main text.
    Text,
    /// The second text.
    Answer,
    /// The tags.
    Tags,
    /// The due date.
    Due,
}

/// An in-app notification that can be clicked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    /// Headline.
    pub title: String,
    /// One more line.
    pub detail: String,
    /// What a click does.
    pub action: ToastAction,
    /// When it goes away by itself (Unix milliseconds); `None`: it stays until dismissed.
    pub expires_ms: Option<i64>,
}

/// What clicking a toast does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastAction {
    /// Open the vault dialog.
    OpenVaultDialog,
    /// Open the calendar.
    OpenCalendar,
    /// Nothing: a click only dismisses it.
    Dismiss,
    /// Undo a mark or unmark operation.
    UndoMark(studyfrog_service::bulk::Snapshot),
    /// Open the PDF viewer for a file.
    OpenPdf(String),
    /// Open the folder of a file on disk.
    OpenFolder(String),
}

/// What a create request produced.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateOutcome {
    /// The vault afterwards.
    pub info: VaultInfo,
    /// The new item.
    pub created: Created,
    /// What could not be added to the board or the calendar.
    pub warnings: Vec<String>,
}

/// Messages of the modals and toasts.
#[derive(Debug, Clone, PartialEq)]
pub enum ModalMsg {
    /// Open the create dialog.
    OpenCreate,
    /// Open the create dialog for stacks, inside the stack that is open.
    OpenCreateStack,
    /// Open the vault dialog.
    OpenVault,
    /// Open the import dialog for the open stack.
    OpenImport,
    /// Start the first-run tour from its card on Home.
    TourStart,
    /// Hide the tour card on Home; the tour stays in Help.
    TourDecline,
    /// Close the open dialog (a Create dialog with something written asks first).
    Close,
    /// Answer the question before closing with 'Keep editing'.
    KeepEditing,
    /// Answer it with 'Discard': close the dialog and lose what was written.
    Discard,
    /// Another kind of item.
    SetKind(NewKind),
    /// The answer type of a question.
    SetAnswerType(crate::view::choice_body::AnswerType),
    /// A field changed.
    Edit(Field, String),
    /// Another parent stack.
    SetParent(Option<String>),
    /// Another referencing stack (optional).
    SetAlsoIn(Option<String>),
    /// "Create another" changed.
    SetAnother(bool),
    /// Choose the priority.
    SetPriority(studyfrog_core::priority::Priority),
    /// Choose the difficulty.
    SetDifficulty(studyfrog_core::event::Difficulty),
    /// A reminder lead time was switched on or off.
    ToggleReminder(u32),
    /// "Create" was activated.
    Submit,
    /// The vault answered.
    Created(Result<Box<CreateOutcome>, String>),
    /// A toast was clicked.
    ToastClicked(usize),
    /// A toast was dismissed.
    ToastDismissed(usize),
    /// Ask to unmark the stack at `folder`.
    AskUnmark(String),
    /// The unmark plan was loaded.
    UnmarkPlanLoaded(Result<Box<studyfrog_service::stack_mark::UnmarkPlan>, String>),
    /// Confirm unmarking the stack at `folder`.
    ConfirmUnmark(String),
    /// Mark an existing normal folder as a stack.
    MarkAsStack(String),
    /// An unmark operation finished.
    Unmarked(
        Result<
            (
                VaultInfo,
                studyfrog_service::bulk::Snapshot,
                studyfrog_service::stack_mark::UnmarkPlan,
            ),
            String,
        >,
    ),
    /// A mark operation finished.
    Marked(Result<(VaultInfo, studyfrog_service::bulk::Snapshot, Created), String>),
    /// Undo a mark or unmark operation.
    UndoMark(studyfrog_service::bulk::Snapshot),
    /// An undo operation finished.
    MarkUndone(Result<VaultInfo, String>),
    /// A normal folder was created.
    CreatedFolder(Result<(VaultInfo, String), String>),
    /// Set whether a folder is a stack folder.
    SetIsStack(bool),
    /// Loaded personalized item-complexity baseline from vault history (EPIC-077).
    ComplexityProfileLoaded(
        Option<studyfrog_report::complexity_baseline::RetentionComplexityProfile>,
    ),
    /// Dismiss the active WIP item-complexity warning for this draft.
    DismissComplexityWarning,
}

pub use crate::modal_form::{problem, stack_labels};
