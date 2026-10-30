//! The whole UI state as plain data. No egui types, no I/O.

use studyfrog_core::settings::AppSettings;
pub use studyfrog_service::session::VaultInfo;

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
}
