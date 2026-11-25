//! Choice answers in rehearsal, graded with the mock exam's rules (STORY-0221). A thin adapter:
//! it builds the exam's answer key and answer, then calls `exam_answer::grade_objective` for the
//! all-or-nothing `correct` flag. The partial fractions of the other schemes come from
//! `choice_scheme` (STORY-0231). Pure.

use studyfrog_core::choice_grading::ChoiceGrading;

use crate::choice_scheme::credited_fraction;
use crate::curriculum_build::AnswerKey;
use crate::exam_answer::{ExamAnswer, format_answer_text, format_expected_text, grade_objective};
use crate::exam_format::ExamItemFormat;

/// A choice item as the grader sees it: the option texts and the 0-based indices of the correct ones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceKey {
    /// Option texts, in the order written.
    pub options: Vec<String>,
    /// Indices of the correct options.
    pub correct_indices: Vec<usize>,
}

/// The outcome of one graded choice answer.
#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceGrade {
    /// True when the chosen options are exactly the correct ones (all-or-nothing).
    pub correct: bool,
    /// True for multiple select (two or more correct options), false for single choice.
    pub multiple: bool,
    /// The correct options, rendered as the mock exam renders them.
    pub expected: String,
    /// The chosen options, rendered as the mock exam renders them. This is the text to record.
    pub given: String,
    /// The fraction of the maximum credited under the scheme, 0 to 1 (STORY-0231).
    pub fraction: f32,
}

/// Grade `picked` (one flag per option, in option order) against `key`, all-or-nothing.
///
/// Single choice uses `SingleSelect`, where exactly one chosen option must be the correct one.
/// Multiple select uses `MultipleSelect` with the points equal to the correct count, so the
/// exam's net partial credit is whole-number and full credit means every correct option chosen
/// and no wrong one: all-or-nothing, as EPIC-072 asks. Returns `None` when the key has no
/// correct option.
pub fn grade_choice(key: &ChoiceKey, picked: &[bool]) -> Option<ChoiceGrade> {
    grade_choice_with(key, picked, ChoiceGrading::AllOrNothing)
}

/// [`grade_choice`] with the credited fraction under `scheme` (STORY-0231). The `correct` flag and
/// the texts are the same under every scheme; only `fraction` differs.
pub fn grade_choice_with(
    key: &ChoiceKey,
    picked: &[bool],
    scheme: ChoiceGrading,
) -> Option<ChoiceGrade> {
    let format = choice_format(key)?;
    let possible = key.correct_indices.len() as u32;
    let answer = exam_answer(format, picked);
    let answer_key = AnswerKey {
        format,
        prompt: String::new(),
        sample_answer: String::new(),
        rubric: Vec::new(),
        correct_indices: key.correct_indices.clone(),
        gap_answers: Vec::new(),
    };
    let graded = grade_objective(format, &answer, &answer_key, &key.options, possible)?;
    Some(ChoiceGrade {
        correct: graded.earned == graded.possible,
        multiple: format == ExamItemFormat::MultipleSelect,
        expected: format_expected_text(&answer_key, &key.options),
        given: format_answer_text(&answer, &key.options),
        fraction: credited_fraction(scheme, picked, &correct_flags(key)),
    })
}

/// The correct options as one flag per option, in option order.
fn correct_flags(key: &ChoiceKey) -> Vec<bool> {
    (0..key.options.len())
        .map(|i| key.correct_indices.contains(&i))
        .collect()
}

/// The exam format for a key: one correct option is single select, more is multiple select.
fn choice_format(key: &ChoiceKey) -> Option<ExamItemFormat> {
    match key.correct_indices.len() {
        0 => None,
        1 => Some(ExamItemFormat::SingleSelect),
        _ => Some(ExamItemFormat::MultipleSelect),
    }
}

/// The exam's answer for the picked flags. A single choice needs exactly one flag set.
fn exam_answer(format: ExamItemFormat, picked: &[bool]) -> ExamAnswer {
    match format {
        ExamItemFormat::MultipleSelect => ExamAnswer::Multiple(picked.to_vec()),
        _ => ExamAnswer::Single(exactly_one(picked)),
    }
}

/// The index of the only set flag, or `None` when no flag or several flags are set.
fn exactly_one(picked: &[bool]) -> Option<usize> {
    let mut set = picked
        .iter()
        .enumerate()
        .filter(|&(_, &on)| on)
        .map(|(i, _)| i);
    let first = set.next()?;
    set.next().is_none().then_some(first)
}
