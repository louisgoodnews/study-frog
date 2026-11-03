//! Side effects `update` asks for, as plain data. The executor performs them and answers with a `Msg`.

use studyfrog_board::ops::BoardOp;
use studyfrog_calendar::session::Session;
use studyfrog_core::plan::RunPlan;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_core::settings::AppSettings;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_rehearsal::input::StepOutput;
use studyfrog_service::rehearse::StepContext;

/// Work to do outside the pure update.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Read the per-user app settings.
    LoadAppSettings,
    /// Ask the user for a file to import cards from.
    PickImportFile,
    /// Read a file and import its cards into a stack with the importer for its extension.
    ImportFile {
        /// Vault-relative stack folder.
        folder: String,
        /// Absolute path of the file.
        path: String,
    },
    /// Make cards from one source text with a registered card type.
    CreateFromType {
        /// Vault-relative stack folder.
        folder: String,
        /// The card type's id.
        type_id: String,
        /// The text the user wrote.
        source: String,
    },
    /// Ask the user for a vault folder with the native dialog.
    PickVaultFolder,
    /// Open (and index) a vault folder.
    OpenVault(String),
    /// Check the open vault for changed files.
    PollVault,
    /// Look for calendar reminders that have come due.
    CheckReminders,
    /// Persist the per-user app settings.
    SaveAppSettings(AppSettings),
    /// List the items of a stack (or the inbox).
    LoadItems(crate::state::Selection),
    /// Count the items of a stack and its sub-stacks by status.
    LoadStatusCounts(crate::state::Selection),
    /// Read every item of the vault as a row of the library list.
    LoadLibrary,
    /// Bulk edits and saved filters of the library.
    Library(crate::library_bulk::LibraryEffect),
    /// Store or read the notifications.
    Notifications(crate::notify::NotifyEffect),
    /// Record how long an item was viewed or worked on.
    RecordView(crate::view_time::ViewRecord),
    /// Count the items a run built from this plan would contain.
    PreviewQueue {
        /// The plan.
        plan: RunPlan,
        /// Presentation mode.
        mode: RunMode,
    },
    /// Create and store a run from this plan.
    StartRun {
        /// The plan.
        plan: RunPlan,
        /// Presentation mode.
        mode: RunMode,
    },
    /// Read an item's content from its file.
    LoadItem(Uid),
    /// Save one engine step (events and, for scheduled runs, the schedule).
    RecordStep {
        /// The run and what it does to the schedule.
        ctx: StepContext,
        /// What the step produced.
        out: StepOutput,
    },
    /// Compare a typed answer with the item's reference.
    ScoreAnswer {
        /// The run the answer belongs to.
        run: Uid,
        /// The item answered.
        item: Uid,
        /// What the user typed.
        answer: String,
        /// Compare by word matching only, whatever is installed.
        words_only: bool,
    },
    /// Replace the rating of an item's latest review and rebuild its schedule.
    Rerate {
        /// The item.
        item: Uid,
        /// The rating the review has now.
        expected: Rating,
        /// The rating it should have.
        to: Rating,
    },
    /// Create a stack, note, card or question.
    CreateItem {
        /// What to create.
        kind: studyfrog_service::create_more::NewKind,
        /// Title (empty: from the text).
        title: String,
        /// Description, body, front or prompt.
        text: String,
        /// Back or reference answer.
        answer: String,
        /// Tags.
        tags: Vec<String>,
        /// The stack it goes into.
        parent: Option<String>,
        /// For a stack: due date and reminders.
        due: Option<studyfrog_service::track::DueDate>,
        /// Priority and difficulty chosen.
        choices: crate::modal_types::Choices,
    },
    /// Read what the home screen shows.
    LoadHome,
    /// Read the assistant's connections.
    LoadAgent,
    /// Save a connection (and its key).
    SaveConnection {
        /// The connection.
        provider: studyfrog_agent::provider::Provider,
        /// Its API key, if it has one.
        key: Option<String>,
    },
    /// Remove a connection and its key.
    RemoveConnection(String),
    /// Allow or forbid sending vault text to a connection.
    AllowConnection(String, bool),
    /// Collect what a wiki request would send.
    PrepareWiki {
        /// The stacks to read.
        folders: Vec<String>,
        /// Pages of a PDF to read as well: path, first and last page (0-based).
        pdf: Option<(String, usize, usize)>,
    },
    /// Ask a model one question.
    AgentChat {
        /// The model.
        provider: studyfrog_agent::provider::Provider,
        /// Its instructions.
        system: String,
        /// The conversation.
        messages: Vec<studyfrog_agent::provider::Message>,
        /// What the answer is for.
        purpose: crate::agent_types::ChatPurpose,
    },
    /// Apply the changes the user accepted.
    ApplyChanges(Vec<studyfrog_agent::wiki::Change>),
    /// Choose the questions of an exam.
    PrepareExam {
        /// The stacks.
        folders: Vec<String>,
        /// How many questions.
        count: usize,
    },
    /// Record exam verdicts as reviews.
    RecordExam(Vec<studyfrog_agent::exam::ExamResult>),
    /// Make a new profile and switch to it.
    CreateProfile(String),
    /// Switch to another profile.
    SwitchProfile(String),
    /// Delete a profile and its settings and keys (not its vault).
    DeleteProfile(String),
    /// Ask where to put a backup of the vault and write it.
    Backup {
        /// Put the API keys in it.
        include_keys: bool,
    },
    /// Ask for a backup file and read what it holds.
    ChooseBackup,
    /// Ask for an empty folder and unpack a backup there.
    Restore {
        /// The backup file.
        archive: String,
        /// Replace the app settings with those of the backup.
        with_settings: bool,
    },
    /// Choose a picture, copy it into the vault and report where it went.
    AddImage(crate::image_ui::ImageTarget),
    /// Read the vault's teachers and subjects.
    LoadSubjects,
    /// Store the vault's teachers and subjects.
    SaveSubjects(studyfrog_config::subjects::SubjectBook),
    /// Find what the item page shows besides the file: the items that link to it (by title) and its familiarity.
    LoadBacklinks(studyfrog_core::uid::Uid, String),
    /// Find items whose title contains this text (the global search).
    SearchItems(String),
    /// Compute the reports from the history.
    BuildReports,
    /// Compute the performance of a scope over a period.
    Explore {
        /// What to look at.
        scope: studyfrog_service::explore::Scope,
        /// How far back.
        period: studyfrog_service::explore::Period,
    },
    /// Measure the achievements and remember new unlocks.
    CheckAchievements,
    /// Read the link graph from the index.
    LoadGraph,
    /// Read all sessions and today's local date.
    LoadSessions,
    /// Read, move or put back the reviews the schedule planned.
    Reviews(crate::planner_reviews::ReviewEffect),
    /// Store a session; id `0` means a new one.
    SaveSession(Session),
    /// Delete a session.
    DeleteSession(Uid),
    /// Skip one occurrence of a session.
    CancelOccurrence(Uid, studyfrog_core::civil::Date),
    /// Read the board (creating the default one on first use).
    LoadBoard,
    /// Apply a board command and store the result.
    BoardCommand(BoardOp),
    /// Read a document for editing.
    LoadDocument(Uid),
    /// Write an edited document, if the file is unchanged since `hash`.
    SaveDocument {
        /// Vault-relative path.
        path: String,
        /// The document to write.
        doc: Doc,
        /// Fingerprint of the file when it was loaded.
        hash: u64,
    },
    /// Find the item a wikilink points to.
    ResolveLink(String),
    /// Read or write the user's help pages.
    Help(crate::help_types::HelpEffect),
    /// Something the PDF viewer needs.
    Pdf(crate::pdf_types::PdfEffect),
    /// Create a card in the stack at `folder`.
    CreateCard {
        /// Vault-relative stack folder.
        folder: String,
        /// Title (may be empty).
        title: String,
        /// Front text.
        front: String,
        /// Back text.
        back: String,
    },
}
