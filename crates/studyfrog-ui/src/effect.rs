//! Side effects `update` asks for, as plain data. The executor performs them and answers with a `Msg`.

use studyfrog_board::ops::BoardOp;
use studyfrog_calendar::session::Session;
use studyfrog_core::civil::Date;
use studyfrog_core::plan::RunPlan;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_core::settings::AppSettings;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_rehearsal::input::StepOutput;
use studyfrog_service::rehearse::StepContext;

use crate::state::Selection;

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
    /// Persist the per-user app settings.
    SaveAppSettings(AppSettings),
    /// List the items of a stack (or the inbox).
    LoadItems(Selection),
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
        /// Title (may be empty for cards and questions).
        title: String,
        /// Description, body, front or prompt.
        text: String,
        /// Back or reference answer.
        answer: String,
        /// Tags.
        tags: Vec<String>,
        /// The stack it goes into.
        parent: Option<String>,
    },
    /// Find items whose title contains this text (the global search).
    SearchItems(String),
    /// Compute the reports from the history.
    BuildReports,
    /// Measure the achievements and remember new unlocks.
    CheckAchievements,
    /// Read the link graph from the index.
    LoadGraph,
    /// Read all sessions and today's local date.
    LoadSessions,
    /// Store a session; id `0` means a new one.
    SaveSession(Session),
    /// Delete a session.
    DeleteSession(Uid),
    /// Skip one occurrence of a session.
    CancelOccurrence(Uid, Date),
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
