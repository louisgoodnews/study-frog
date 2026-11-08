//! The exam rules of the engine (`docs/specs/exam-mode.md`, section 3): written answers only, one way.
//! Checked during validation, so a refused input leaves the session unchanged.

use studyfrog_core::run::RunMode;

use crate::input::{Action, StepError};
use crate::session::Session;

/// True when the session is an exam.
pub fn is_exam(s: &Session) -> bool {
    s.run.mode == RunMode::Exam
}

/// Why an exam refuses `action`, if it does. Other runs are never refused here.
pub fn exam_refusal(s: &Session, action: &Action) -> Option<StepError> {
    if !is_exam(s) {
        return None;
    }
    match action {
        Action::Flip | Action::Skip | Action::Rate(_) | Action::Mark(_) => {
            Some(StepError::NotInExam)
        }
        Action::SubmitAnswer(text) if text.trim().is_empty() => Some(StepError::EmptyAnswer),
        _ => None,
    }
}

/// True for the modes in which a free-text answer can be submitted.
pub fn takes_answers(mode: RunMode) -> bool {
    matches!(mode, RunMode::Answer | RunMode::Exam)
}
