//! Grading a mock exam (STORY-0187): each answer is judged against its question's rubric, by the local model
//! when one is given and by the keyword check otherwise. A question's points are its points times the share
//! of rubric points the answer makes; the results are summed by the grade scale. Pure: the model call is
//! passed in.

use studyfrog_agent::exam_grade::{Ask, grade_answer, is_counted, share_met};
use studyfrog_rehearsal::curriculum_build::AnswerKey;
use studyfrog_rehearsal::grade::{ExamResult, exam_result};
use studyfrog_rehearsal::mock_exam::{MockExam, all_questions};

use super::MockExamUi;

/// Share of the points at which the exam is passed by default (German "Ausreichend": 50 %). The student
/// can set another pass mark for the exam (STORY-0187).
pub const DEFAULT_PASS_MARK: f32 = 50.0;

/// The pass mark of an exam in percent: the student's setting, or the default.
pub fn pass_mark_of(ui: &MockExamUi) -> f32 {
    ui.pass_mark.unwrap_or(DEFAULT_PASS_MARK)
}

/// The pass mark as the student may set it: whole percent from 1 to 100; anything else is clamped.
pub fn clamp_pass_mark(pct: f32) -> f32 {
    if pct.is_finite() {
        pct.round().clamp(1.0, 100.0)
    } else {
        DEFAULT_PASS_MARK
    }
}

/// One question after grading.
#[derive(Debug, Clone, PartialEq)]
pub struct Graded {
    /// The question's topic.
    pub topic: String,
    /// The question.
    pub prompt: String,
    /// What the student wrote.
    pub answer: String,
    /// The reference answer.
    pub sample_answer: String,
    /// Each rubric point: `Some(true)` when the answer makes it, `Some(false)` when it does not, and `None`
    /// when the keyword check could not test it ("not checked by word overlap").
    pub points_made: Vec<(String, Option<bool>)>,
    /// Points available.
    pub possible: u32,
    /// Points earned.
    pub earned: u32,
    /// False when no rubric point could be checked: the question is left out of the exam's total.
    pub counted: bool,
    /// True when the local model judged the answer; false when the keyword check did.
    pub from_model: bool,
    /// What the student should know about this grade, if anything.
    pub caveat: Option<String>,
}

/// Grade every question of `exam` by the keyword check, in order. An answer missing from `answers` earns nothing.
pub fn grade(exam: &MockExam, keys: &[AnswerKey], answers: &[String]) -> Vec<Graded> {
    grade_with(exam, keys, answers, None)
}

/// Grade every question of `exam` against its key, in order: by the model when `ask` is given (a question
/// the model cannot grade falls back to the keyword check), otherwise by the keyword check alone.
pub fn grade_with(
    exam: &MockExam,
    keys: &[AnswerKey],
    answers: &[String],
    mut ask: Option<&mut Ask<'_>>,
) -> Vec<Graded> {
    all_questions(exam)
        .into_iter()
        .zip(keys)
        .enumerate()
        .map(|(i, (q, key))| {
            let answer = answers.get(i).cloned().unwrap_or_default();
            let judged = grade_answer(
                &q.prompt,
                &key.sample_answer,
                &key.rubric,
                &answer,
                ask.as_deref_mut(),
            );
            let points_made: Vec<(String, Option<bool>)> = judged
                .criteria
                .iter()
                .map(|c| (c.criterion.clone(), c.checked.then_some(c.met)))
                .collect();
            let possible = u32::from(q.points.0);
            Graded {
                topic: q.topic.clone(),
                prompt: q.prompt.clone(),
                answer,
                sample_answer: key.sample_answer.clone(),
                points_made,
                possible,
                earned: (possible as f32 * share_met(&judged.criteria)).round() as u32,
                counted: is_counted(&judged.criteria),
                from_model: judged.from_model,
                caveat: judged.caveat,
            }
        })
        .collect()
}

/// Points available for a question in the total: none when it is not counted.
pub fn counted_possible(g: &Graded) -> u32 {
    if g.counted { g.possible } else { 0 }
}

/// The exam's total: points, percentage and grade at this pass mark. Questions that are not counted are
/// left out.
pub fn total(graded: &[Graded], pass_mark: f32) -> ExamResult {
    let earned: Vec<u32> = graded.iter().map(|g| g.earned).collect();
    let possible: Vec<u32> = graded.iter().map(counted_possible).collect();
    exam_result(&earned, &possible, pass_mark)
}

/// Points earned and available per topic, in the order the topics first appear.
pub fn breakdown(graded: &[Graded]) -> Vec<(String, u32, u32)> {
    let mut out: Vec<(String, u32, u32)> = Vec::new();
    for g in graded {
        match out.iter_mut().find(|(t, _, _)| *t == g.topic) {
            Some(row) => {
                row.1 += g.earned;
                row.2 += counted_possible(g);
            }
            None => out.push((g.topic.clone(), g.earned, counted_possible(g))),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use studyfrog_rehearsal::curriculum::chunk_markdown;
    use studyfrog_rehearsal::curriculum_build::{assemble, plan, templates};

    #[test]
    fn a_full_answer_earns_all_points_and_an_empty_one_none() {
        let chunks = chunk_markdown(
            "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\n",
        );
        let qs = templates(&chunks, plan(2));
        let (exam, keys) = assemble("K", &qs);
        let full: Vec<String> = keys
            .iter()
            .map(|k| format!("{} {}", k.sample_answer, k.rubric.join(" ")))
            .collect();
        let g = grade(&exam, &keys, &full);
        assert!(g.iter().all(|q| q.earned == q.possible), "{g:?}");
        let none = grade(&exam, &keys, &[]);
        assert!(none.iter().all(|q| q.earned == 0));
        assert_eq!(total(&none, DEFAULT_PASS_MARK).percent.unwrap_or(0.0), 0.0);
    }

    #[test]
    fn with_no_model_every_question_is_graded_by_the_keyword_check() {
        let chunks = chunk_markdown("# Reliabilität\nReliabilität: consistency of measurement.\n");
        let (exam, keys) = assemble("K", &templates(&chunks, plan(2)));
        let g = grade(&exam, &keys, &[]);
        assert!(g.iter().all(|q| !q.from_model && q.caveat.is_some()));
    }

    #[test]
    fn a_model_that_judges_every_point_met_earns_every_point() {
        let chunks = chunk_markdown("# Reliabilität\nReliabilität: consistency of measurement.\n");
        let (exam, keys) = assemble("K", &templates(&chunks, plan(2)));
        let mut ask = |_: &str, _: &str| {
            let items: Vec<String> = keys
                .iter()
                .flat_map(|k| k.rubric.iter())
                .map(|c| {
                    format!("{{\"criterion_text\": {c:?}, \"met\": true, \"feedback\": \"ok\"}}")
                })
                .collect();
            Ok(format!("{{\"criteria_results\": [{}]}}", items.join(",")))
        };
        let g = grade_with(
            &exam,
            &keys,
            &["x".to_owned(), "y".to_owned()],
            Some(&mut ask),
        );
        assert!(
            g.iter().all(|q| q.from_model && q.earned == q.possible),
            "{g:?}"
        );
    }

    #[test]
    fn the_breakdown_groups_by_topic_in_order() {
        let g = |topic: &str, e: u32, p: u32| Graded {
            topic: topic.to_owned(),
            prompt: String::new(),
            answer: String::new(),
            sample_answer: String::new(),
            points_made: Vec::new(),
            possible: p,
            earned: e,
            counted: true,
            from_model: false,
            caveat: None,
        };
        let b = breakdown(&[g("A", 2, 4), g("B", 1, 1), g("A", 3, 3)]);
        assert_eq!(b, vec![("A".to_owned(), 5, 7), ("B".to_owned(), 1, 1)]);
    }
}
