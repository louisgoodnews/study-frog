//! The commands of the screens that had only `nav`: achievements, graph, assistant, settings, match,
//! folders, and opening a PDF in the viewer. Each one reads its screen's data or changes one thing on it.

/// What the achievements screen is asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AchievementsCmd {
    /// The entries the screen lists (under its current filter).
    List,
    /// Show only these entries (a view setting; nothing is saved).
    Filter(AchFilter),
}

/// The filter words: `all`, `global`, or `stack FOLDER` (the stack's achievements, resolved in the plan).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AchFilter {
    /// Everything.
    All,
    /// The vault's own achievements.
    Global,
    /// One stack's achievements, by its folder.
    Stack(String),
}

/// What `assistant ...` asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantCmd {
    /// The assistant's state: its tab, connections and chat.
    State,
    /// Send this prompt to the connected model (refused when none is connected).
    Send(String),
}

/// What `settings ...` asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCmd {
    /// The sections and the values they show.
    Read,
    /// Say during a run when an answer takes far longer than usual (`on`) or not.
    SlowAnswer(bool),
}

/// What `match ...` asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchCmd {
    /// The match state: tiles, pairs left, mismatches, time.
    State,
    /// A new match game from the items of the vault.
    Start,
}

/// What `folders ...` asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoldersCmd {
    /// The files and folders the screen lists.
    List,
    /// Select a file or folder (vault-relative) and expand it.
    Open(String),
}

/// The screens' commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenCmd {
    /// Achievements.
    Achievements(AchievementsCmd),
    /// The link graph: its nodes and edges.
    Graph,
    /// The assistant.
    Assistant(AssistantCmd),
    /// Settings.
    Settings(SettingsCmd),
    /// Match mode.
    Match(MatchCmd),
    /// Folders.
    Folders(FoldersCmd),
    /// Open this vault-relative PDF in the viewer (page count and find results are answered).
    PdfOpen(String),
}
