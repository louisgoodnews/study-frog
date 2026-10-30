//! Everything that can happen: user input and results of effects.

use studyfrog_core::settings::AppSettings;

use crate::state::{ItemRow, Screen, Selection, VaultInfo};

/// An event for [`update`](crate::update::update).
#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    /// The app settings were loaded (or defaulted, with the reason if reading failed).
    Started {
        /// Loaded settings.
        app: AppSettings,
        /// Why the settings file could not be used.
        error: Option<String>,
    },
    /// The vault path field changed.
    PathEdited(String),
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
    /// The app settings were saved (or not).
    AppSettingsSaved(Result<(), String>),
    /// A stack (or the inbox) was chosen in the browser.
    Select(Selection),
    /// The title filter changed.
    SearchEdited(String),
    /// The items of the selection were loaded.
    ItemsLoaded(Result<Vec<ItemRow>, String>),
    /// The new-stack title changed.
    NewStackEdited(String),
    /// "Create stack" was activated.
    CreateStackClicked,
    /// A field of the new-card form changed.
    DraftEdited(DraftField, String),
    /// "Add card" was activated.
    AddCardClicked,
    /// A stack or card was created (or not); carries the refreshed vault.
    Created(Result<VaultInfo, String>),
}

/// Which field of the new-card form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftField {
    /// The optional title.
    Title,
    /// The front.
    Front,
    /// The back.
    Back,
}
