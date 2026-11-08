//! Rehearsal run data (see `docs/specs/rehearsal-run.md`). Transitions live in `studyfrog-rehearsal`.

use crate::plan::RunPlan;
use crate::time::UnixMs;
use crate::uid::Uid;

/// How items are presented and answered in a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunMode {
    /// Show the prompt, user flips and rates.
    Flip,
    /// User types an answer that is scored by the SLM.
    Answer,
    /// A strict exam: written answers only, one way, graded at the end (`docs/specs/exam-mode.md`).
    Exam,
}

/// Lifecycle of a run: `Idle -> Running <-> Paused -> Finished | Aborted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunState {
    /// Queue fixed, not started.
    Idle,
    /// Timer running.
    Running,
    /// Paused; paused time is excluded from latency.
    Paused,
    /// Ended normally.
    Finished,
    /// Ended early or recovered after a crash.
    Aborted,
}

/// A rehearsal session over a fixed queue.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// Run identity.
    pub uid: Uid,
    /// Presentation mode.
    pub mode: RunMode,
    /// How the queue was built and what the run does to the schedule.
    pub plan: RunPlan,
    /// Items to rehearse, in order.
    pub queue: Vec<Uid>,
    /// Current lifecycle state.
    pub state: RunState,
    /// Wall-clock start, set when the run starts.
    pub started_at: Option<UnixMs>,
}

/// Persisted name of a mode.
pub fn run_mode_name(mode: RunMode) -> &'static str {
    match mode {
        RunMode::Flip => "flip",
        RunMode::Answer => "answer",
        RunMode::Exam => "exam",
    }
}

/// Inverse of [`run_mode_name`].
pub fn run_mode_from_name(name: &str) -> Option<RunMode> {
    match name {
        "flip" => Some(RunMode::Flip),
        "answer" => Some(RunMode::Answer),
        "exam" => Some(RunMode::Exam),
        _ => None,
    }
}

/// Persisted name of a state.
pub fn run_state_name(state: RunState) -> &'static str {
    match state {
        RunState::Idle => "idle",
        RunState::Running => "running",
        RunState::Paused => "paused",
        RunState::Finished => "finished",
        RunState::Aborted => "aborted",
    }
}

/// Inverse of [`run_state_name`].
pub fn run_state_from_name(name: &str) -> Option<RunState> {
    match name {
        "idle" => Some(RunState::Idle),
        "running" => Some(RunState::Running),
        "paused" => Some(RunState::Paused),
        "finished" => Some(RunState::Finished),
        "aborted" => Some(RunState::Aborted),
        _ => None,
    }
}
