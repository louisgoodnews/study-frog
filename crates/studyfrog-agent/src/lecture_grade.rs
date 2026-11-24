//! The teacher's grade of a turned-in answer (STORY-0210, TASK-0292): the prompt that asks the model to
//! judge each rubric point and summarise, the parser that reads the reply, and a deterministic keyword
//! check used when the reply is missing or cannot be read. The score is always recomputed here from the
//! criteria, so the model cannot pick its own total. Pure: nothing here calls a model.

use serde::Deserialize;

use crate::lecture_prompt::{PromptMessage, PromptRole};
use crate::rubric_eval::{CriterionResult, fallback_evaluate, json_span};

/// The most points an answer can score. Points are the share of checked rubric points met, out of 100.
pub const GRADE_MAX: u32 = 100;

/// A graded answer, ready to be recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grade {
    /// Points out of [`GRADE_MAX`].
    pub points: u32,
    /// The letter for the points: A (90 and above) to F.
    pub label: &'static str,
    /// Whether each rubric point is met, in rubric order.
    pub criteria: Vec<CriterionResult>,
    /// The teacher's summary feedback for the student.
    pub summary: String,
    /// True when the model made the judgements.
    pub from_model: bool,
    /// What the student should know about this grade, when the keyword check was used and why.
    pub caveat: Option<String>,
}

/// The system and user messages that ask the teacher to grade `answer` against the rubric.
pub fn grade_messages(
    teacher: &str,
    title: &str,
    prompt: &str,
    rubric: &[String],
    answer: &str,
) -> Vec<PromptMessage> {
    let points: Vec<String> = rubric.iter().map(|point| format!("- {point}")).collect();
    let system = format!(
        "You are {teacher}, the teacher in a study classroom. Grade the student's answer to the task \
         \"{title}\": {prompt}\n\nRubric points:\n{}\n\nFor each rubric point, mark it met only when the \
         answer makes that point, in any words. Give one sentence of feedback for each point. Then write a \
         summary of two or three sentences for the student. Judge only the answer. Reply with one JSON \
         object and nothing else, like {{\"criteria\": [{{\"criterion\": \"<rubric point exactly as \
         listed>\", \"met\": true, \"feedback\": \"<one sentence>\"}}], \"summary\": \"<summary>\"}}.",
        points.join("\n"),
    );
    vec![
        PromptMessage {
            role: PromptRole::System,
            content: system,
        },
        PromptMessage {
            role: PromptRole::User,
            content: format!("Student's answer:\n{answer}"),
        },
    ]
}

/// The JSON reply as the wire carries it.
#[derive(Deserialize)]
struct Wire {
    criteria: Vec<WireCriterion>,
    summary: String,
}

#[derive(Deserialize)]
struct WireCriterion {
    criterion: String,
    met: bool,
    #[serde(default)]
    feedback: String,
}

/// Reads the teacher's reply as a grade. Each rubric point is matched by its text (ignoring case); a
/// point the reply does not mention is counted as not met. Refused when there is no JSON object, the
/// summary is blank, or the reply names none of the rubric points. The caller then uses the keyword check.
pub fn parse_grade(reply: &str, rubric: &[String]) -> Result<Grade, String> {
    let json = json_span(reply).ok_or_else(|| "the reply had no JSON object".to_owned())?;
    let wire: Wire =
        serde_json::from_str(json).map_err(|e| format!("the reply is not valid: {e}"))?;
    let summary = wire.summary.trim();
    if summary.is_empty() {
        return Err("the reply has no summary".to_owned());
    }
    let found: Vec<Option<&WireCriterion>> = rubric
        .iter()
        .map(|point| {
            wire.criteria
                .iter()
                .find(|w| w.criterion.trim().eq_ignore_ascii_case(point.trim()))
        })
        .collect();
    if found.iter().all(Option::is_none) {
        return Err("the reply names none of the rubric points".to_owned());
    }
    let criteria: Vec<CriterionResult> = rubric
        .iter()
        .zip(found)
        .map(|(point, w)| criterion_from(point, w))
        .collect();
    Ok(grade_of(criteria, summary.to_owned(), true, None))
}

/// The result of one rubric point, from the model's judgement of it, or not met when there is none.
fn criterion_from(point: &str, judgement: Option<&WireCriterion>) -> CriterionResult {
    match judgement {
        Some(w) => CriterionResult {
            criterion: point.to_owned(),
            met: w.met,
            checked: true,
            feedback: w.feedback.clone(),
        },
        None => CriterionResult {
            criterion: point.to_owned(),
            met: false,
            checked: true,
            feedback: "Not assessed by the model.".to_owned(),
        },
    }
}

/// Grades `answer` from the model's `reply`. When the reply is an error or cannot be read, the keyword
/// check grades the answer and the caveat says why the model was not used.
pub fn grade_answer(reply: Result<&str, &str>, answer: &str, rubric: &[String]) -> Grade {
    let failure = match reply {
        Ok(text) => match parse_grade(text, rubric) {
            Ok(grade) => return grade,
            Err(why) => why,
        },
        Err(why) => why.to_owned(),
    };
    keyword_grade(
        answer,
        rubric,
        Some(format!(
            "The teacher could not grade this answer ({failure}); the keyword check is shown."
        )),
    )
}

/// Grades `answer` by word overlap with each rubric point, without a model. `caveat` says why no model
/// was used; the check's own limits are added when it has none of its own.
pub fn keyword_grade(answer: &str, rubric: &[String], caveat: Option<String>) -> Grade {
    let report = fallback_evaluate(answer, true, rubric);
    let checked = report.criteria.iter().filter(|c| c.checked).count();
    let met = report
        .criteria
        .iter()
        .filter(|c| c.checked && c.met)
        .count();
    let summary = if checked == 0 {
        "No rubric point could be checked by word overlap.".to_owned()
    } else {
        format!("The keyword check found {met} of {checked} checked rubric points in the answer.")
    };
    grade_of(report.criteria, summary, false, caveat.or(report.caveats))
}

fn grade_of(
    criteria: Vec<CriterionResult>,
    summary: String,
    from_model: bool,
    caveat: Option<String>,
) -> Grade {
    let points = points_of(&criteria);
    Grade {
        points,
        label: label_of(points),
        criteria,
        summary,
        from_model,
        caveat,
    }
}

/// Points out of [`GRADE_MAX`]: the share of the checked rubric points that are met, rounded down.
/// Points that were not checked do not count either way; when none was checked the score is 0.
pub fn points_of(criteria: &[CriterionResult]) -> u32 {
    let checked = u32::try_from(criteria.iter().filter(|c| c.checked).count()).unwrap_or(u32::MAX);
    if checked == 0 {
        return 0;
    }
    let met =
        u32::try_from(criteria.iter().filter(|c| c.checked && c.met).count()).unwrap_or(u32::MAX);
    met * GRADE_MAX / checked
}

/// The letter for a score out of [`GRADE_MAX`]: A from 90, B from 80, C from 70, D from 60, otherwise F.
pub fn label_of(points: u32) -> &'static str {
    match points {
        p if p >= 90 => "A",
        p if p >= 80 => "B",
        p if p >= 70 => "C",
        p if p >= 60 => "D",
        _ => "F",
    }
}
