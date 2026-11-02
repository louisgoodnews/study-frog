//! The data of the dialogs and toasts, and the small pure functions about it.

use studyfrog_core::civil::parse_date;
use studyfrog_service::create_more::{Created, NewKind};
use studyfrog_store::overview::VaultOverview;

use crate::state::VaultInfo;

/// Which modal is open.
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
}

/// What clicking a toast does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastAction {
    /// Open the vault dialog.
    OpenVaultDialog,
    /// Open the calendar.
    OpenCalendar,
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
    /// Close the open dialog.
    Close,
    /// Another kind of item.
    SetKind(NewKind),
    /// A field changed.
    Edit(Field, String),
    /// Another parent stack.
    SetParent(Option<String>),
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
}

/// The stacks as (folder, "Parent / Child" label), in tree order.
pub fn stack_labels(o: &VaultOverview) -> Vec<(String, String)> {
    let title_of = |folder: &str| {
        o.stacks
            .iter()
            .find(|s| s.folder == folder)
            .map(|s| s.title.clone())
    };
    o.stacks
        .iter()
        .map(|s| {
            let mut chain = vec![s.title.clone()];
            let mut up = s.parent.clone();
            while let Some(p) = up {
                chain.push(title_of(&p).unwrap_or_else(|| p.clone()));
                up = o
                    .stacks
                    .iter()
                    .find(|x| x.folder == p)
                    .and_then(|x| x.parent.clone());
            }
            chain.reverse();
            (s.folder.clone(), chain.join(" / "))
        })
        .collect()
}

/// Why the form cannot be submitted yet, in words for the user.
pub fn problem(f: &CreateForm) -> Option<&'static str> {
    let blank = |s: &str| s.trim().is_empty();
    match f.kind {
        NewKind::Stack | NewKind::Note if blank(&f.title) => Some("Give it a title."),
        NewKind::Card if blank(&f.text) || blank(&f.answer) => {
            Some("A card needs a front and a back.")
        }
        NewKind::Question if blank(&f.text) || blank(&f.answer) => {
            Some("A question needs a prompt and a reference answer.")
        }
        NewKind::Stack if !blank(&f.due) && parse_date(f.due.trim()).is_none() => {
            Some("The due date must look like 2026-12-31.")
        }
        NewKind::Stack => None,
        _ if f.parent.is_none() => Some("Choose the stack it goes into."),
        _ => None,
    }
}
