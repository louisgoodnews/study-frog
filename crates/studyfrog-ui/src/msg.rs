//! Everything that can happen: user input and results of effects.

use crate::chrome::ChromeMsg;
use crate::home_ui::HomeMsg;
use crate::kanban::BoardMsg;
use crate::modal::ModalMsg;
use crate::planner::PlanMsg;
use crate::reading_test::ReadingTestMsg;

pub mod browse;
pub mod editor;
pub mod graph;
pub mod import;
pub mod rehearsal;
pub mod reports;
pub mod vault;
pub use browse::MsgBrowse;
pub use editor::MsgEditor;
pub use graph::MsgGraph;
pub use import::MsgImport;
pub use rehearsal::MsgRehearsal;
pub use reports::MsgReports;
pub use vault::MsgVault;

/// An event for [`update`](crate::update::update).
#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    /// Vault, startup, navigation and settings messages.
    Vault(MsgVault),
    /// Import and picture messages.
    Import(MsgImport),
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
    /// A message of the subjects section.
    Subjects(crate::subjects_ui::SubjectsMsg),
    /// A message of the Console screen.
    Console(crate::console_ui::ConsoleMsg),
    /// Stack browser and item list messages.
    Browse(MsgBrowse),
    /// A message of the case-study view.
    Case(crate::setup::case::CaseMsg),
    /// A message of the math drill.
    Math(crate::setup::math::MathMsg),
    /// A message of the mock exam.
    MockExam(crate::setup::mock_exam::MockExamMsg),
    /// The learn-mode editor (STORY-0198).
    Learn(crate::learn_editor::LearnMsg),
    /// A scheduling switch of the vault.
    Srs(crate::app_settings_ui::SrsMsg),
    /// A message of the memory game.
    Memory(crate::setup::memory::MemoryMsg),
    /// Run setup and rehearsal messages.
    Rehearsal(MsgRehearsal),
    /// Editing an item during rehearsal.
    RehearsalEdit(crate::rehearsal_edit::RehearsalEditMsg),
    /// Exam mode: grading and its result.
    Exam(crate::exam_ui::ExamMsg),
    /// Document editor messages.
    Editor(MsgEditor),
    /// Link graph canvas messages.
    Graph(MsgGraph),
    /// Something happened on the home screen or in the stack browser's shortcuts.
    Home(HomeMsg),
    /// Something happened in a dialog or a toast.
    Modal(ModalMsg),
    /// Something happened in the window chrome (sidebar, menus, search).
    Chrome(ChromeMsg),
    /// Report and achievement results.
    Reports(MsgReports),
    /// The application's logs folder: open it, or how that went (STORY-0122).
    Logs(crate::logs_ui::LogsMsg),
    /// Something happened on the achievements screen or its stack summary.
    Achievements(crate::achievements::AchievementsMsg),
    /// Something happened on the calendar screen.
    Plan(PlanMsg),
    /// Messages of the homework screen.
    Homework(crate::homework_msg::HomeworkMsg),
    /// Something happened on the board screen.
    Board(BoardMsg),
    /// Something happened in the reading-speed test.
    ReadingTest(ReadingTestMsg),
    /// Something happened on the library list.
    Library(crate::library_ui::LibraryMsg),
    /// An appearance or start-up setting changed.
    Appearance(crate::appearance_ui::AppearanceMsg),
    /// Something happened on the help screen.
    Help(crate::help_types::HelpMsg),
    /// A tab of the settings screen was chosen or stepped to.
    SettingsTab(crate::app_settings_ui::SettingsTabMsg),
    /// File explorer folders screen.
    Folders(crate::folders_types::FoldersMsg),
    /// The active-recall workflows (EPIC-035).
    Recall(crate::recall_types::RecallMsg),
    /// FrogCanvas 2D spatial board messages.
    Canvas(crate::canvas_types::CanvasMsg),
    /// Match Mode tile pairing recall drill (EPIC-053).
    Match(crate::match_ui::MatchMsg),
    /// A message of the classroom screen.
    Classroom(crate::classroom_types::ClassroomMsg),
    /// A message of the first-run setup wizard (STORY-0212).
    Wizard(crate::view::wizard::types::WizardMsg),
}
pub use crate::{edit_form::EditAction, state::DraftField};
