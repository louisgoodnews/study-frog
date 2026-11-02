//! The whole UI state as plain data. No egui types, no I/O.

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
    /// The assistant: connections, wiki proposals and exams.
    Agent,
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
    /// A PDF file open for reading.
    Pdf,
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
    /// How many items of the stack, with its sub-stacks, are in each status, as last loaded.
    pub status_counts: Vec<(studyfrog_core::status::Status, u32)>,
    /// Title filter.
    pub search: String,
    /// Card being written.
    pub draft: CardDraft,
    /// The card type of the new-card form (`None`: an ordinary front and back).
    pub card_type: Option<String>,
    /// The file to import cards from, as typed.
    pub import_path: String,
}

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
    /// Seconds on the shell's monotonic clock, refreshed every frame. Time enters the pure update
    /// only through this field.
    pub now: f64,
    /// The wall clock in Unix milliseconds, set from the system clock at the start of each frame.
    pub wall_ms: i64,
    /// The document open for editing.
    pub editor: Option<Editor>,
    /// The visit to the item on the page that is being timed.
    pub view: Option<crate::view_time::ViewTracker>,
    /// The link graph.
    pub graph: Option<GraphState>,
    /// The calendar.
    pub planner: Planner,
    /// The kanban board.
    pub kanban: Kanban,
    /// The open dialog.
    pub modal: Option<Modal>,
    /// Clickable in-app notifications.
    pub toasts: Vec<Toast>,
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
    /// The reading-speed test on the settings screen.
    pub reading_test: ReadingTest,
    /// The PDFs of the vault, for the library list.
    pub pdf_library: crate::pdf_library_types::PdfLibrary,
    /// The PDF open in the viewer.
    pub pdf: Option<crate::pdf_types::PdfView>,
}
