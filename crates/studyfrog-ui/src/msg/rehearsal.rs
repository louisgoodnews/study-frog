//! Run setup and rehearsal messages.

use crate::run_state::{ItemContent, Scored};
use crate::setup::SetupEdit;
use studyfrog_core::{review::Rating, run::Run, uid::Uid};
use studyfrog_rehearsal::input::Action;

/// Run setup and rehearsal messages.
#[derive(Debug, Clone, PartialEq)]
pub enum MsgRehearsal {
    /// A field of the run setup form changed.
    Setup(SetupEdit),
    /// Another step of the run setup.
    SetupStep(crate::setup::SetupStep),
    /// The number of items a run would contain (or why that could not be counted).
    PreviewReady(Result<usize, String>),
    /// "Start rehearsal" was activated.
    StartRunClicked,
    /// The run was created and stored.
    RunReady(Result<Run, String>),
    /// The user did something in the run (key press or button).
    RunAction(Action),
    /// A choice item's options were picked or submitted (STORY-0222).
    Choice(crate::run_update::choice::ChoiceMsg),
    /// The answer field changed.
    AnswerEdited(String),
    /// An item's content was loaded.
    ItemLoaded(Uid, Result<ItemContent, String>),
    /// A typed answer was compared with its reference.
    AnswerScored(Uid, Result<Scored, String>),
    /// "Apply" was activated for a suggested rating.
    ApplySuggestion {
        /// Item UID.
        item: Uid,
        /// Expected current rating.
        expected: Rating,
        /// Suggested target rating.
        to: Rating,
    },
    /// A rating was replaced (or not).
    Rerated(Uid, Result<(), String>),
    /// A step was saved (or not).
    StepRecorded(Result<(), String>),
    /// "Back to setup" after a run.
    LeaveRun,
}
