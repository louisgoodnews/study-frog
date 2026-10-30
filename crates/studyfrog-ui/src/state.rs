//! The whole UI state as plain data. No egui types, no I/O.

use studyfrog_core::settings::AppSettings;
pub use studyfrog_service::session::VaultInfo;

use crate::setup::Rehearse;
pub use studyfrog_store::overview::ItemRow;

/// Which screen is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    /// Choose a vault.
    #[default]
    Welcome,
    /// Stacks and items of the open vault.
    Library,
    /// Set up and run a rehearsal.
    Rehearse,
    /// Vault and application settings.
    Settings,
}

/// Severity of a notice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    /// Something worked.
    Info,
    /// Something failed.
    Error,
}

/// A message for the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    /// Severity.
    pub kind: NoticeKind,
    /// Text shown to the user.
    pub text: String,
}

/// What the library screen shows on its right-hand side.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Selection {
    /// Nothing chosen yet.
    #[default]
    Nothing,
    /// The stack in this vault-relative folder.
    Stack(String),
    /// Items outside every stack.
    Inbox,
}

/// The card being written in the "new card" form.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CardDraft {
    /// Optional title (derived from the front when empty).
    pub title: String,
    /// Prompt side.
    pub front: String,
    /// Answer side.
    pub back: String,
}

/// State of the stack browser.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Browser {
    /// Which stack (or the inbox) is open.
    pub selection: Selection,
    /// Items of the selection, as last loaded.
    pub items: Vec<ItemRow>,
    /// Title filter.
    pub search: String,
    /// Title for a stack about to be created.
    pub new_stack: String,
    /// Card being written.
    pub draft: CardDraft,
}

/// Everything the views render.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct State {
    /// Current screen.
    pub screen: Screen,
    /// Per-user settings (last and recent vaults).
    pub app: AppSettings,
    /// Text in the vault path field.
    pub path_input: String,
    /// The open vault, if any.
    pub vault: Option<VaultInfo>,
    /// Status-bar message.
    pub notice: Option<Notice>,
    /// A vault is being opened.
    pub opening: bool,
    /// A change check is running in the background.
    pub polling: bool,
    /// The stack browser.
    pub browser: Browser,
    /// The rehearse screen.
    pub rehearse: Rehearse,
    /// Seconds on the shell's monotonic clock, refreshed every frame. Time enters the pure update
    /// only through this field.
    pub now: f64,
}
