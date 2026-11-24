//! Screens of the application, as plain data.

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
    /// The homework planner: assignments, due dates and checklists.
    Homework,
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
    /// Teach a simulated lecture: a chat with a teacher, simulated students and you.
    Classroom,
}
