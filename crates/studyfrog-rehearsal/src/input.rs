//! Engine inputs, outputs and errors.

use std::fmt;

use studyfrog_core::event::{Difficulty, Latency, RunEvent};
use studyfrog_core::review::Rating;

/// What the user (or shell) did.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Begin the run; the first item is shown automatically.
    Start,
    /// Reveal the answer without typing one.
    Flip,
    /// Pass on the item without rating it.
    Skip,
    /// Submit a free-text answer (answer mode only).
    SubmitAnswer(String),
    /// Rate the revealed item; the next item is shown automatically.
    Rate(Rating),
    /// Mark how hard the item on screen is (before or after the answer is revealed).
    Mark(Difficulty),
    /// Pause the timer.
    Pause,
    /// Resume after a pause.
    Resume,
    /// Leave the run early.
    Abort,
    /// Time has passed and nothing else happened; a countdown may run out.
    Tick,
}

/// An action with the monotonic time (milliseconds since run start) at which it happened.
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    /// Monotonic milliseconds; must not decrease between inputs.
    pub t_ms: u32,
    /// What happened.
    pub action: Action,
}

/// What a successful step produced.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StepOutput {
    /// New events in order.
    pub events: Vec<RunEvent>,
    /// Latency of the item interaction this step completed, if any (paused time excluded).
    pub latency: Option<Latency>,
}

/// Why an input was refused. The session is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepError {
    /// The run has not been started.
    NotStarted,
    /// The run is already running.
    AlreadyStarted,
    /// The run is finished or aborted.
    Ended,
    /// The run is paused; resume first.
    Paused,
    /// The run is not paused.
    NotPaused,
    /// The item has not been shown or has already been revealed.
    NotAwaitingReveal,
    /// Rating needs a revealed item.
    NotRevealed,
    /// There is no item on screen to mark.
    NoItemShown,
    /// Free-text answers only exist in answer mode.
    WrongMode,
    /// The timestamp is earlier than the previous input's.
    TimeWentBackwards,
    /// An exam takes only written answers: no flip, skip, rating or mark.
    NotInExam,
    /// An exam answer must not be empty.
    EmptyAnswer,
}

/// Human-readable description.
pub fn describe_step_error(e: StepError) -> &'static str {
    match e {
        StepError::NotStarted => "the run has not been started",
        StepError::AlreadyStarted => "the run is already started",
        StepError::Ended => "the run has ended",
        StepError::Paused => "the run is paused",
        StepError::NotPaused => "the run is not paused",
        StepError::NotAwaitingReveal => "the item is not waiting to be revealed",
        StepError::NotRevealed => "the item has not been revealed yet",
        StepError::NoItemShown => "there is no item on screen",
        StepError::WrongMode => "free-text answers need an answer-mode run",
        StepError::TimeWentBackwards => "input time is earlier than the previous input",
        StepError::NotInExam => "an exam takes only written answers: no flip, skip, rating or mark",
        StepError::EmptyAnswer => "write an answer before submitting it",
    }
}

impl fmt::Display for StepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(describe_step_error(*self))
    }
}

impl std::error::Error for StepError {}
