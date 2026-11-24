//! Run, rehearsal and scoring effects.

use {
    studyfrog_core::{plan::RunPlan, review::Rating, run::RunMode, uid::Uid},
    studyfrog_rehearsal::input::StepOutput,
    studyfrog_service::rehearse::StepContext,
};

/// Run, rehearsal and scoring effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectRehearsal {
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
    /// Replace the rating of an item's latest review and rebuild its schedule.
    Rerate {
        /// The item.
        item: Uid,
        /// The rating the review has now.
        expected: Rating,
        /// The rating it should have.
        to: Rating,
    },
}
