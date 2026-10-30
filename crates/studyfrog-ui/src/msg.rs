//! Everything that can happen: user input and results of effects.

use studyfrog_core::settings::AppSettings;

use studyfrog_core::review::Rating;
use studyfrog_core::run::Run;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::document::{LoadedDoc, Saved};
use studyfrog_store::overview::LinkGraph;

use crate::kanban::BoardMsg;
use crate::planner::PlanMsg;
use crate::reading_test::ReadingTestMsg;
use crate::run_state::{ItemContent, Scored};
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
    /// A typed answer was compared with its reference.
    AnswerScored(Uid, Result<Scored, String>),
    /// "Apply" was activated for a suggested rating.
    ApplySuggestion {
        /// The item.
        item: Uid,
        /// The rating given in the run.
        expected: Rating,
        /// The suggested rating.
        to: Rating,
    },
    /// A rating was replaced (or not).
    Rerated(Uid, Result<(), String>),
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
    /// The link graph was read from the index.
    GraphLoaded(Result<LinkGraph, String>),
    /// A frame of the graph canvas of this size (pixels) is being drawn.
    GraphFrame {
        /// Canvas width.
        width: f32,
        /// Canvas height.
        height: f32,
    },
    /// The canvas was dragged by this many pixels.
    GraphPan(f32, f32),
    /// The canvas was scrolled; multiply the zoom by this factor.
    GraphZoom(f32),
    /// The pointer is over this node (or none).
    GraphHover(Option<usize>),
    /// "Re-run layout" was activated.
    GraphRestart,
    /// A node was clicked.
    GraphClick(usize),
    /// Something happened on the calendar screen.
    Plan(PlanMsg),
    /// Something happened on the board screen.
    Board(BoardMsg),
    /// Something happened in the reading-speed test.
    ReadingTest(ReadingTestMsg),
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
