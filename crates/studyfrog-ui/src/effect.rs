//! Side effects `update` asks for, as plain data. The executor performs them and answers with a `Msg`.

pub mod assistant;
pub mod backup;
pub mod browse;
pub mod editor;
pub mod import;
pub mod planner;
pub mod rehearsal;
pub mod reports;
pub mod vault;
pub use assistant::EffectAssistant;
pub use backup::EffectBackup;
pub use browse::EffectBrowse;
pub use editor::EffectEditor;
pub use import::EffectImport;
pub use planner::EffectPlanner;
pub use rehearsal::EffectRehearsal;
pub use reports::EffectReports;
pub use vault::EffectVault;

/// Work to do outside the pure update.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Vault, settings, reminders and subject book effects.
    Vault(EffectVault),
    /// Import, card creation and picture effects.
    Import(EffectImport),
    /// Work of the homework screen (save, advance, tick a step).
    Homework(crate::homework_msg::HomeworkEffect),
    /// Read the homework items for the calendar's deadline markers.
    LoadHomework,
    /// Stack browser, search and item lookup effects.
    Browse(EffectBrowse),
    /// Bulk edits and saved filters of the library.
    Library(crate::library_bulk::LibraryEffect),
    /// Store or read the notifications.
    Notifications(crate::notify::NotifyEffect),
    /// Run, rehearsal and scoring effects.
    Rehearsal(EffectRehearsal),
    /// Make or save a mock exam from the notes of a folder.
    MockExam(crate::setup::mock_exam::MockExamEffect),
    /// The Console's work: the transcript written to the vault, the scheduled jobs (STORY-0152, STORY-0151).
    Console(crate::console_ui::ConsoleEffect),
    /// Read or save learn modes for the editor (STORY-0198).
    Learn(crate::learn_editor::LearnEffect),
    /// Exam mode: grade a finished exam.
    Exam(crate::exam_ui::ExamEffect),
    /// Work on a typed or case answer: score it, or record a case check's score.
    Answer(crate::run_state::AnswerEffect),
    /// Write prompt files.
    Prompts(crate::prompts_ui::PromptsEffect),
    /// Assistant, wiki and exam effects.
    Assistant(EffectAssistant),
    /// The assistant's connections, staged proposals and undo entries.
    Agent(crate::agent_mcp_types::AgentEffect),
    /// Backup, restore and profile effects.
    Backup(EffectBackup),
    /// Report, explorer and achievement effects.
    Reports(EffectReports),
    /// Document and link effects.
    Editor(EffectEditor),
    /// Calendar sessions, timetable and board effects.
    Planner(EffectPlanner),
    /// Read, move or put back the reviews the schedule planned.
    Reviews(crate::planner_reviews::ReviewEffect),
    /// Read or write the user's help pages.
    Help(crate::help_types::HelpEffect),
    /// Something the PDF viewer needs.
    Pdf(crate::pdf_types::PdfEffect),
    /// Marking or unmarking stack folders.
    StackMark(crate::stack_mark_ui::StackMarkEffect),
    /// File explorer operations.
    Folders(crate::folders_types::FolderEffect),
    /// Diary operations.
    Diary(crate::diary_types::DiaryEffect),
    /// Operations for editing items during rehearsal.
    RehearsalEdit(crate::rehearsal_edit::RehearsalEditEffect),
    /// A classroom turn or session change, run through the service.
    Classroom(crate::classroom_types::ClassroomEffect),
}
