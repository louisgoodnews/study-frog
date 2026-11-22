//! Grading a mock exam (STORY-0187): each answer is checked against its question's rubric, its points are
//! the share of rubric points it makes, and the results are summed by the grade scale. Pure.

use studyfrog_agent::rubric_eval::fallback_evaluate;
use studyfrog_rehearsal::curriculum_build::AnswerKey;
use studyfrog_rehearsal::grade::{ExamResult, exam_result};
use studyfrog_rehearsal::mock_exam::{MockExam, all_questions};

/// Share of the points at which the exam is passed (German "Ausreichend": 50 %).
pub const PASS_MARK: f32 = 50.0;

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
    /// Each rubric point, and whether the answer makes it.
    pub points_made: Vec<(String, bool)>,
    /// Points available.
    pub possible: u32,
    /// Points earned.
    pub earned: u32,
}

/// Grade every question of `exam` against its key, in order. An answer missing from `answers` earns nothing.
pub fn grade(exam: &MockExam, keys: &[AnswerKey], answers: &[String]) -> Vec<Graded> {
    all_questions(exam)
        .into_iter()
        .zip(keys)
        .enumerate()
        .map(|(i, (q, key))| {
            let answer = answers.get(i).cloned().unwrap_or_default();
            let report = fallback_evaluate(&answer, true, &key.rubric);
            let points_made: Vec<(String, bool)> = report
                .criteria
                .iter()
                .map(|c| (c.criterion.clone(), c.met))
                .collect();
            let met = points_made.iter().filter(|(_, m)| *m).count();
            let share = if points_made.is_empty() {
                0.0
            } else {
                met as f32 / points_made.len() as f32
            };
            let possible = u32::from(q.points.0);
            Graded {
                topic: q.topic.clone(),
                prompt: q.prompt.clone(),
                answer,
                sample_answer: key.sample_answer.clone(),
                points_made,
                possible,
                earned: (possible as f32 * share).round() as u32,
            }
        })
        .collect()
}

/// The exam's total: points, percentage and grade.
pub fn total(graded: &[Graded]) -> ExamResult {
    let earned: Vec<u32> = graded.iter().map(|g| g.earned).collect();
    let possible: Vec<u32> = graded.iter().map(|g| g.possible).collect();
    exam_result(&earned, &possible, PASS_MARK)
}

/// Points earned and available per topic, in the order the topics first appear.
pub fn breakdown(graded: &[Graded]) -> Vec<(String, u32, u32)> {
    let mut out: Vec<(String, u32, u32)> = Vec::new();
    for g in graded {
        match out.iter_mut().find(|(t, _, _)| *t == g.topic) {
            Some(row) => {
                row.1 += g.earned;
                row.2 += g.possible;
            }
            None => out.push((g.topic.clone(), g.earned, g.possible)),
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
        assert_eq!(total(&none).percent.unwrap_or(0.0), 0.0);
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
        };
        let b = breakdown(&[g("A", 2, 4), g("B", 1, 1), g("A", 3, 3)]);
        assert_eq!(b, vec![("A".to_owned(), 5, 7), ("B".to_owned(), 1, 1)]);
    }
}
