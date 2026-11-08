//! Messages, effects and state of the active-recall workflows (EPIC-035): syncing a note's inline
//! cards, the contrastive drill, the focus timer and the occlusion editor. Plain data.

use studyfrog_core::uid::Uid;
use studyfrog_service::create_more::Created;
use studyfrog_service::inline_sync::{SyncOutcome, SyncPlan};

/// The "Extract / Sync cards" panel of a note's page.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtractPanel {
    /// The note the panel is about.
    pub note: Option<Uid>,
    /// What a sync would do, once planned.
    pub plan: Option<SyncPlan>,
    /// Planning or syncing is under way.
    pub busy: bool,
    /// Why the last step failed.
    pub error: Option<String>,
    /// What the last sync did: created and updated.
    pub done: Option<(usize, usize)>,
}

/// The state of the active-recall workflows.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RecallState {
    /// The extraction panel (open when `note` is set).
    pub extract: ExtractPanel,
    /// The focus timer.
    pub focus: crate::focus_timer::Focus,
    /// The contrastive drill, while open.
    pub contrast: Option<crate::contrast::Contrast>,
}

/// Something happened in an active-recall workflow.
#[derive(Debug, Clone, PartialEq)]
pub enum RecallMsg {
    /// "Extract / Sync cards" on a note's page: plan the sync of this note with this body.
    ExtractPreview(Uid, String),
    /// The plan arrived.
    ExtractPlanned(Result<Box<SyncPlan>, String>),
    /// "Sync" in the panel.
    ExtractSync,
    /// The sync is done.
    ExtractSynced(Result<Box<SyncOutcome>, String>),
    /// Close the panel.
    ExtractClose,
    /// The focus timer.
    Focus(crate::focus_ui::FocusMsg),
    /// The contrastive drill.
    Contrast(crate::contrast::ContrastMsg),
    /// Make an image occlusion card in the stack `0` from the vault picture `1`.
    OcclusionCard(String, String),
    /// The occlusion card was made (or not).
    OcclusionMade(Result<Box<(studyfrog_service::session::VaultInfo, Created)>, String>),
}

/// Work for the worker.
#[derive(Debug, Clone, PartialEq)]
pub enum RecallEffect {
    /// Plan the sync of a note with this body.
    PlanExtract(Uid, String),
    /// Carry out a sync.
    ApplyExtract(Box<SyncPlan>),
    /// Record a finished focus block.
    LogFocus(studyfrog_service::focus::FocusDone),
    /// Load the items of a contrastive drill.
    LoadContrast(Vec<Uid>),
    /// Keep the distinction of a contrastive drill.
    SaveContrast(Box<crate::contrast::ContrastSave>),
    /// Make an image occlusion card in a stack from a vault picture.
    CreateOcclusion(String, String),
}
