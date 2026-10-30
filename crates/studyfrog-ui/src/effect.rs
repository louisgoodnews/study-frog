//! Side effects `update` asks for, as plain data. The executor performs them and answers with a `Msg`.

use studyfrog_core::plan::RunPlan;
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
    /// Read the link graph from the index.
    LoadGraph,
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
    /// Create a stack with this title.
    CreateStack(String),
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
