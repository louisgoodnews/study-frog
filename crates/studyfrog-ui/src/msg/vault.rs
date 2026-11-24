//! Vault, startup, navigation and settings messages.

use crate::state::{Screen, VaultInfo};
use studyfrog_core::settings::{AnswerEngine, AppSettings};

/// Vault, startup, navigation and settings messages.
#[derive(Debug, Clone, PartialEq)]
pub enum MsgVault {
    /// The app settings were loaded (or defaulted, with the reason if reading failed).
    Started {
        /// Loaded settings.
        app: AppSettings,
        /// Why the settings file could not be used.
        error: Option<String>,
    },
    /// The vault path field changed.
    PathEdited(String),
    /// A card type was chosen in the new-card form (`None`: an ordinary card).
    CardTypeChosen(Option<String>),
    /// "Choose folder" was activated.
    BrowseVaultClicked,
    /// The folder dialog closed: the chosen folder, or `None` when cancelled or unavailable.
    VaultFolderPicked(Option<String>),
    /// "Open vault" was activated.
    OpenVaultClicked,
    /// A recent vault was chosen.
    OpenRecent(String),
    /// The result of opening a vault.
    VaultOpened(Result<VaultInfo, String>),
    /// A navigation button was pressed.
    Navigate(Screen),
    /// "Close vault" was pressed.
    CloseVault,
    /// The status-bar message was dismissed.
    DismissNotice,
    /// About once a second: check the vault for changes.
    Tick,
    /// The result of a change check: `None` when nothing changed.
    VaultPolled(Result<Option<VaultInfo>, String>),
    /// Calendar and homework reminders that came due (each is reported once).
    RemindersDue(Result<studyfrog_calendar::reminders::DueAll, String>),
    /// The app settings were saved (or not).
    AppSettingsSaved(Result<(), String>),
    /// The answer-comparison choice on the settings screen changed.
    SetAnswerEngine(AnswerEngine),
    /// The user touched the keyboard, the mouse or the screen.
    Activity,
    /// Recording how long items are viewed was switched on or off.
    SetTrackViews(bool),
}
