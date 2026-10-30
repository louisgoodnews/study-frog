//! Everything that can happen: user input and results of effects.

use studyfrog_core::settings::AppSettings;

use studyfrog_core::run::Run;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::document::{LoadedDoc, Saved};

use crate::run_state::ItemContent;
use crate::setup::SetupEdit;
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
    /// A field of the run setup form changed.
    Setup(SetupEdit),
    /// The number of items a run would contain (or why that could not be counted).
    PreviewReady(Result<usize, String>),
    /// "Start rehearsal" was activated.
    StartRunClicked,
    /// The run was created and stored.
    RunReady(Result<Run, String>),
    /// The user did something in the run (key press or button).
    RunAction(Action),
    /// The answer field changed.
    AnswerEdited(String),
    /// An item's content was loaded.
    ItemLoaded(Uid, Result<ItemContent, String>),
    /// A step was saved (or not).
    StepRecorded(Result<(), String>),
    /// "Back to setup" after a run.
    LeaveRun,
    /// An item was chosen for editing.
    OpenItem(Uid),
    /// A document was read for editing.
    DocumentLoaded(Uid, Result<LoadedDoc, String>),
    /// A field of the editor changed.
    Edit(EditAction),
    /// "Save" was activated.
    SaveClicked,
    /// The document was saved (or not).
    Saved(Result<Saved, String>),
    /// "Revert" was activated.
    RevertClicked,
    /// "Reload from disk" was activated.
    ReloadClicked,
    /// "Back" was activated in the editor.
    CloseEditor,
    /// A wikilink in the preview was clicked.
    OpenLink(String),
    /// The item a wikilink points to (`None`: no item has that title).
    LinkResolved(String, Result<Option<Uid>, String>),
}

/// A change in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditAction {
    /// The title field.
    Title(String),
    /// The tags field.
    Tags(String),
    /// A text area, by index.
    Field(usize, String),
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
