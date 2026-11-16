//! Distractor multiple-choice exam view and automated grading.
//!
//! Complies with Invariant P1: zero inherent impl blocks; domain logic is
//! implemented entirely in pure free functions.

use studyfrog_rehearsal::distractor::MultipleChoiceQuestion;

pub use crate::view::exam_mcq_view::{render_mcq_view, render_mcq_view_with_palette};

/// State for the multiple-choice exam view and automated grading.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct McqState {
    /// The current question being presented, if any.
    pub current_question: Option<MultipleChoiceQuestion>,
    /// Index of the currently selected choice in `all_choices`.
    pub selected_choice: Option<usize>,
    /// Whether the user has submitted their choice and the answer is revealed.
    pub is_revealed: bool,
    /// Count of correctly answered questions.
    pub score: usize,
    /// Total count of answered questions.
    pub total_answered: usize,
}

/// Messages for multiple-choice interaction and state transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McqMsg {
    /// Choose an option by its 0-based index.
    SelectChoice(usize),
    /// Submit current choice or advance to next question.
    SubmitOrNext,
    /// Load a new question.
    LoadQuestion(MultipleChoiceQuestion),
    /// Reset the exam session and counters.
    Reset,
}

/// Create a new, blank MCQ state.
pub fn new_mcq_state() -> McqState {
    McqState::default()
}

/// Load a question into MCQ state, resetting selection and reveal status.
pub fn mcq_load_question(state: &mut McqState, question: MultipleChoiceQuestion) {
    state.current_question = Some(question);
    state.selected_choice = None;
    state.is_revealed = false;
}

/// Select a choice index. Ignored if already revealed.
pub fn mcq_select_choice(state: &mut McqState, choice_index: usize) {
    if state.is_revealed {
        return;
    }
    if let Some(q) = &state.current_question {
        if choice_index < q.all_choices.len() {
            state.selected_choice = Some(choice_index);
        }
    } else {
        state.selected_choice = Some(choice_index);
    }
}

/// Submit current choice (grading and toggling reveal) or advance to next.
pub fn mcq_submit_or_next(state: &mut McqState) -> Option<bool> {
    if !state.is_revealed {
        let question = state.current_question.as_ref()?;
        let selected = state.selected_choice?;
        let is_correct = selected == question.correct_index;
        state.is_revealed = true;
        state.total_answered += 1;
        if is_correct {
            state.score += 1;
        }
        Some(is_correct)
    } else {
        state.is_revealed = false;
        state.selected_choice = None;
        state.current_question = None;
        None
    }
}

/// Process an `McqMsg` transition on `McqState`.
pub fn update_mcq(state: &mut McqState, msg: McqMsg) -> Option<bool> {
    match msg {
        McqMsg::SelectChoice(idx) => {
            mcq_select_choice(state, idx);
            None
        }
        McqMsg::SubmitOrNext => mcq_submit_or_next(state),
        McqMsg::LoadQuestion(q) => {
            mcq_load_question(state, q);
            None
        }
        McqMsg::Reset => {
            *state = new_mcq_state();
            None
        }
    }
}
