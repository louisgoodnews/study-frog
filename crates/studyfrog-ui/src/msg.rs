//! Everything that can happen: user input and results of effects.

use studyfrog_core::settings::{AnswerEngine, AppSettings};

use studyfrog_core::review::Rating;
use studyfrog_core::run::Run;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::achievements::AchievementReport;
use studyfrog_service::document::{LoadedDoc, Saved};
use studyfrog_service::reports::ReportBundle;
use studyfrog_store::overview::LinkGraph;

use crate::chrome::ChromeMsg;
use crate::home_ui::HomeMsg;
use crate::import_ui::ImportOutcome;
use crate::kanban::BoardMsg;
use crate::modal::ModalMsg;
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
    /// A card type was chosen in the new-card form (`None`: an ordinary card).
    CardTypeChosen(Option<String>),
    /// The path of the file to import changed.
    ImportPathEdited(String),
    /// "Browse" was activated next to the import path.
    BrowseImportClicked,
    /// The file dialog closed: the chosen file, or `None`.
    ImportFilePicked(Option<String>),
    /// "Import cards" was activated.
    ImportClicked,
    /// Cards were made by an import or a card type (or not).
    Imported(Result<ImportOutcome, String>),
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
    /// A message of the assistant screen.
    Agent(crate::agent_types::AgentMsg),
    /// The prompts for an AI.
    Prompts(crate::prompts_ui::PromptsMsg),
    /// The PDF viewer.
    Pdf(crate::pdf_types::PdfMsg),
    /// A message about profiles.
    Profile(crate::profile_ui::ProfileMsg),
    /// A message of the backup section.
    Backup(crate::backup_ui::BackupMsg),
    /// A message of the diary screen.
    Diary(crate::diary_msg::DiaryMsg),
    /// A message of the performance explorer.
    Explorer(crate::explorer_ui::ExplorerMsg),
    /// The user asked to add a picture to a text area.
    PickImage(crate::image_ui::ImageTarget),
    /// A picture was copied into the vault (or the user cancelled).
    ImageAdded(crate::image_ui::ImageTarget, crate::image_ui::Added),
    /// A message of the subjects section.
    Subjects(crate::subjects_ui::SubjectsMsg),
    /// The items that link to an open item.
    BacklinksLoaded(
        studyfrog_core::uid::Uid,
        Result<Vec<studyfrog_store::overview::ItemRow>, String>,
    ),
    /// How familiar the user is with an open item, over time.
    FamCurveLoaded(
        studyfrog_core::uid::Uid,
        Result<Box<studyfrog_service::fam_curve::ItemCurve>, String>,
    ),
    /// Reminders that came due (each is reported once).
    RemindersDue(Result<Vec<studyfrog_calendar::reminders::DueReminder>, String>),
    /// The app settings were saved (or not).
    AppSettingsSaved(Result<(), String>),
    /// A stack (or the inbox) was chosen in the browser.
    Select(Selection),
    /// The title filter changed.
    SearchEdited(String),
    /// The items of the selection were loaded.
    ItemsLoaded(Result<Vec<ItemRow>, String>),
    /// The status counts of the selected stack were loaded.
    StatusCountsLoaded(Result<Vec<(studyfrog_core::status::Status, u32)>, String>),
    /// A field of the new-card form changed.
    DraftEdited(DraftField, String),
    /// "Add card" was activated.
    AddCardClicked,
    /// A stack or card was created (or not); carries the refreshed vault.
    Created(Result<VaultInfo, String>),
    /// A field of the run setup form changed.
    Setup(SetupEdit),
    /// Another step of the run setup.
    SetupStep(crate::setup::SetupStep),
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
    /// Save now (the editor saves by itself; this is for tests and the keyboard).
    SaveClicked,
    /// "Keep mine" was activated after the file changed outside StudyFrog.
    KeepMineClicked,
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
    /// Something happened on the home screen or in the stack browser's shortcuts.
    Home(HomeMsg),
    /// Something happened in a dialog or a toast.
    Modal(ModalMsg),
    /// Something happened in the window chrome (sidebar, menus, search).
    Chrome(ChromeMsg),
    /// The reports were computed.
    ReportsBuilt(Result<ReportBundle, String>),
    /// The achievements were measured.
    AchievementsChecked(Result<AchievementReport, String>),
    /// Something happened on the achievements screen or its stack summary.
    Achievements(crate::achievements::AchievementsMsg),
    /// Something happened on the calendar screen.
    Plan(PlanMsg),
    /// Something happened on the board screen.
    Board(BoardMsg),
    /// The answer-comparison choice on the settings screen changed.
    SetAnswerEngine(AnswerEngine),
    /// Something happened in the reading-speed test.
    ReadingTest(ReadingTestMsg),
    /// Something happened on the library list.
    Library(crate::library_ui::LibraryMsg),
    /// The user touched the keyboard, the mouse or the screen.
    Activity,
    /// Recording how long items are viewed was switched on or off.
    SetTrackViews(bool),
    /// An appearance or start-up setting changed.
    Appearance(crate::appearance_ui::AppearanceMsg),
    /// Something happened on the help screen.
    Help(crate::help_types::HelpMsg),
    /// A tab of the settings screen was chosen or stepped to.
    SettingsTab(crate::app_settings_ui::SettingsTabMsg),
    /// File explorer folders screen.
    Folders(crate::folders_types::FoldersMsg),
}

pub use crate::edit_form::EditAction;

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
