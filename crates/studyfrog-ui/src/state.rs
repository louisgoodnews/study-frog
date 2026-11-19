//! The whole UI state as plain data. No egui types, no I/O.

use studyfrog_calendar::homework::HomeworkItem;
use studyfrog_core::settings::AppSettings;
pub use studyfrog_service::session::VaultInfo;

use studyfrog_service::achievements::AchievementReport;
use studyfrog_service::home::HomeSummary;
use studyfrog_service::reports::ReportBundle;

use crate::chrome::Chrome;
use crate::editor::Editor;
use crate::graph::GraphState;
use crate::kanban::Kanban;
use crate::modal::{Modal, Toast};
use crate::planner::Planner;
use crate::reading_test::ReadingTest;
use crate::setup::Rehearse;
pub use studyfrog_store::overview::ItemRow;

/// Which screen is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    /// Choose a vault.
    #[default]
    Welcome,
    /// What is ready today and what happened lately.
    Home,
    /// Physical filesystem hierarchy on disk.
    Folders,
    /// The assistant: connections, wiki proposals and exams.
    Agent,
    /// The standard prompts for an AI: copy, save or run them.
    Prompts,
    /// Stacks and items of the open vault.
    Library,
    /// Set up and run a rehearsal.
    Rehearse,
    /// Vault and application settings.
    Settings,
    /// A document open for editing.
    Editor,
    /// The wikilink graph.
    Graph,
    /// Planned study sessions.
    Calendar,
    /// The kanban board.
    Board,
    /// Achievements and their progress.
    Achievements,
    /// Retention, answer times and the review forecast.
    Reports,
    /// Self-assessment reflections, ratings, and diary notes.
    Diary,
    /// A PDF file open for reading.
    Pdf,
    /// The help pages.
    Help,
    /// Control commands and scripts typed by hand.
    Console,
    /// FrogCanvas 2D spatial concept board.
    Canvas,
    /// Match Mode tile pairing recall drill.
    Match,
    /// Homework assignments, by due date.
    Homework,
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

pub use crate::state_browser::{Browser, CardDraft, DraftField, Selection};

/// What the user can pick from the registered extensions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Extensions {
    /// Card types: (id, name, hint).
    pub card_types: Vec<(String, String, String)>,
    /// Importers: (id, name, file extensions).
    pub importers: Vec<(String, String, Vec<String>)>,
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
    /// The item the notice with this text opens (the notice shows an Open button while it is the one showing).
    pub notice_link: Option<(String, studyfrog_core::uid::Uid)>,
    /// A vault is being opened.
    pub opening: bool,
    /// A change check is running in the background.
    pub polling: bool,
    /// Ticks since the vault opened, to space out the reminder checks.
    pub reminder_ticks: u32,
    /// The stack browser.
    pub browser: Browser,
    /// The rehearse screen.
    pub rehearse: Rehearse,
    /// Seconds on the shell's monotonic clock, refreshed every frame.
    pub now: f64,
    /// The wall clock in Unix milliseconds, set from the system clock at the start of each frame.
    pub wall_ms: i64,
    /// The document open for editing.
    pub editor: Option<Editor>,
    /// The physical filesystem explorer.
    pub folders: crate::folders_types::FoldersState,
    /// The list of the library.
    pub library: crate::library_ui::LibraryState,
    /// The visit to the item on the page that is being timed.
    pub view: Option<crate::view_time::ViewTracker>,
    /// The link graph.
    pub graph: Option<GraphState>,
    /// The calendar.
    pub planner: Planner,
    /// Homework assignments shown on the planner screen.
    pub homework: Vec<HomeworkItem>,
    /// The help screen.
    pub help: crate::help_types::HelpState,
    /// Where the user has been.
    pub history: crate::history::History,
    /// The import and export dialog.
    pub transfer: crate::transfer_ui::TransferState,
    /// The kanban board.
    pub kanban: Kanban,
    /// The open dialog.
    pub modal: Option<Modal>,
    /// Clickable in-app notifications.
    pub toasts: Vec<Toast>,
    /// A message to apply right after the one being handled.
    pub replay: Option<Box<crate::msg::Msg>>,
    /// A standard prompt being run through the assistant.
    pub prompt_run: Box<crate::prompt_run::PromptRun>,
    /// The Prompts tab: category, search and picked options.
    pub prompts_view: crate::prompts_ui::PromptsView,
    /// Sidebar, menus, search and notifications.
    pub chrome: Chrome,
    /// The registered importers and card types.
    pub extensions: Extensions,
    /// The home screen's numbers (also the stack badges in the library).
    pub home: Option<HomeSummary>,
    /// The assistant screen.
    pub agent: crate::agent_types::AgentState,
    /// The user profiles.
    pub profiles: crate::profile_ui::ProfileState,
    /// Backing up and restoring.
    pub backup: crate::backup_ui::BackupState,
    /// The teachers and subjects of the vault.
    pub subjects: crate::subjects_ui::SubjectsState,
    /// The latest reports.
    pub reports: Option<ReportBundle>,
    /// The performance explorer of the reports screen.
    pub explorer: crate::explorer_ui::ExplorerState,
    /// The latest achievement check.
    pub achievements: Option<AchievementReport>,
    /// Which achievements the achievements screen lists.
    pub achievements_filter: crate::achievements::Filter,
    /// The report was asked for by a view (the stack page) and not yet delivered.
    pub achievements_wanted: bool,
    /// The reading-speed test on the settings screen.
    pub reading_test: ReadingTest,
    /// The PDFs of the vault, for the library list.
    pub pdf_library: crate::pdf_library_types::PdfLibrary,
    /// The PDF open in the viewer.
    pub pdf: Option<crate::pdf_types::PdfView>,
    /// The diary screen and configuration.
    pub diary: crate::diary_types::DiaryState,
    /// The Console screen.
    pub console: crate::console_ui::ConsoleState,
    /// The active-recall workflows (EPIC-035).
    pub recall: Box<crate::recall_types::RecallState>,
    /// FrogCanvas 2D spatial concept board.
    pub canvas: Option<crate::canvas_types::CanvasState>,
    /// Split-screen PDF navigation and highlight overlay (EPIC-051).
    pub split_pdf: crate::pdf_types::SplitPdfView,
    /// Match Mode tile pairing recall drill (EPIC-053).
    pub match_state: Option<crate::match_ui::MatchUiState>,
}
