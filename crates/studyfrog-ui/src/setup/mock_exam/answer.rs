//! Structured student answer initialization and update helpers for mock exams (STORY-0207). Pure.

use studyfrog_rehearsal::exam_answer::{ExamAnswer, blank_answer};
use studyfrog_rehearsal::mock_exam::{MockExam, all_questions};

use super::MockExamUi;

/// Initialize blank [`ExamAnswer`] slots matching each question's format and option/gap count.
pub fn init_answers(exam: &MockExam) -> Vec<ExamAnswer> {
    all_questions(exam).into_iter().map(blank_answer).collect()
}

/// Update a free-text answer (`OpenQuestion`, `Calculation`, `CaseScenario`).
pub fn set_text(ui: MockExamUi, idx: usize, text: String) -> MockExamUi {
    if ui.locked {
        return ui;
    }
    let mut answers = ui.answers.clone();
    if let Some(slot) = answers.get_mut(idx) {
        *slot = ExamAnswer::Text(text);
    }
    MockExamUi { answers, ..ui }
}

/// Update one gap inside a `FillInGaps` answer.
pub fn set_gap(ui: MockExamUi, q_idx: usize, gap_idx: usize, text: String) -> MockExamUi {
    if ui.locked {
        return ui;
    }
    let mut answers = ui.answers.clone();
    if let Some(ExamAnswer::Gaps(gaps)) = answers.get_mut(q_idx)
        && let Some(slot) = gaps.get_mut(gap_idx)
    {
        *slot = text;
    }
    MockExamUi { answers, ..ui }
}

/// Select one radio option for a `SingleSelect` question.
pub fn set_single(ui: MockExamUi, q_idx: usize, opt_idx: usize) -> MockExamUi {
    if ui.locked {
        return ui;
    }
    let mut answers = ui.answers.clone();
    if let Some(slot) = answers.get_mut(q_idx) {
        *slot = ExamAnswer::Single(Some(opt_idx));
    }
    MockExamUi { answers, ..ui }
}

/// Toggle one checkbox option for a `MultipleSelect` question.
pub fn toggle_multiple(ui: MockExamUi, q_idx: usize, opt_idx: usize, on: bool) -> MockExamUi {
    if ui.locked {
        return ui;
    }
    let mut answers = ui.answers.clone();
    if let Some(ExamAnswer::Multiple(flags)) = answers.get_mut(q_idx)
        && let Some(slot) = flags.get_mut(opt_idx)
    {
        *slot = on;
    }
    MockExamUi { answers, ..ui }
}
