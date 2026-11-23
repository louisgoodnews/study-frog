//! Grading one written exam answer against its rubric (STORY-0187). The local model judges each rubric point
//! from the student's answer; the keyword check stands in when no model is given or its reply cannot be
//! read, and says so. The model call is passed in, so this module does no I/O.

use serde::Deserialize;

use crate::rubric_eval::{CriterionResult, fallback_evaluate, json_span};

/// The model as the grader asks it: the system instructions and the user's message, in; the reply, out.
pub type Ask<'a> = dyn FnMut(&str, &str) -> Result<String, String> + 'a;

/// One answer's rubric result.
#[derive(Debug, Clone, PartialEq)]
pub struct AnswerGrade {
    /// Whether each rubric point is made, in rubric order.
    pub criteria: Vec<CriterionResult>,
    /// True when the local model made the judgements.
    pub from_model: bool,
    /// What the student should know about this grade (why the keyword check was used, or its limits).
    pub caveat: Option<String>,
}

/// Share of the checked rubric points made, 0.0 to 1.0. Points not checked by word overlap do not count
/// either way. An answer with no checked point earns nothing.
pub fn share_met(criteria: &[CriterionResult]) -> f32 {
    let checked: Vec<&CriterionResult> = criteria.iter().filter(|c| c.checked).collect();
    if checked.is_empty() {
        return 0.0;
    }
    checked.iter().filter(|c| c.met).count() as f32 / checked.len() as f32
}

/// Whether the answer can be graded at all: at least one rubric point was checked. An answer that is
/// not counted is left out of the exam's total rather than scored 0.
pub fn is_counted(criteria: &[CriterionResult]) -> bool {
    criteria.iter().any(|c| c.checked)
}

/// Grade `answer` to a question. With a model, it judges each rubric point; without one, or when its reply
/// cannot be read, the keyword check grades the answer.
pub fn grade_answer(
    question: &str,
    sample: &str,
    rubric: &[String],
    answer: &str,
    ask: Option<&mut Ask<'_>>,
) -> AnswerGrade {
    let Some(ask) = ask else {
        return keyword(answer, rubric, None);
    };
    let system = grade_system(question, sample, rubric);
    let user = format!("Student's answer:\n{answer}");
    match ask(&system, &user).and_then(|reply| parse_criteria(&reply, rubric)) {
        Ok(criteria) => AnswerGrade {
            criteria,
            from_model: true,
            caveat: None,
        },
        Err(e) => keyword(
            answer,
            rubric,
            Some(format!(
                "The local model could not grade this answer ({e}); the keyword check is shown."
            )),
        ),
    }
}

/// The keyword grade, with its caveat (the check's own limits, or why the model was not used).
fn keyword(answer: &str, rubric: &[String], note: Option<String>) -> AnswerGrade {
    let report = fallback_evaluate(answer, true, rubric);
    AnswerGrade {
        criteria: report.criteria,
        from_model: false,
        caveat: note.or(report.caveats),
    }
}

/// The instructions for the model: the question and the reference answer, the rubric points to judge, and
/// the JSON shape of the reply.
pub fn grade_system(question: &str, sample: &str, rubric: &[String]) -> String {
    let mut out = format!(
        "You grade a student's written answer to an exam question. Judge the student's answer only; the \
         reference answer shows what a full answer contains.\n\nQuestion: {question}\nReference answer: \
         {sample}\n\nFor each rubric point below, mark it met only when the student's answer makes that \
         point, in any words. Give one sentence of feedback that quotes the words which show it, or says \
         what is missing.\n\nRubric points:\n"
    );
    for point in rubric {
        out.push_str(&format!("- {point}\n"));
    }
    out.push_str(
        "\nEnd with ONE JSON object on its own, like {\"criteria_results\": [{\"criterion_text\": \
         \"<rubric point exactly as listed>\", \"met\": true, \"feedback\": \"<one sentence>\"}]}. Give every \
         rubric point once.",
    );
    out
}

#[derive(Deserialize)]
struct WireReply {
    criteria_results: Vec<WireCriterion>,
}

#[derive(Deserialize)]
struct WireCriterion {
    criterion_text: String,
    met: bool,
    #[serde(default)]
    feedback: String,
}

/// Read the model's reply: its JSON object, with each rubric point matched by its text (ignoring case). A
/// point the model did not mention is not met.
pub fn parse_criteria(reply: &str, rubric: &[String]) -> Result<Vec<CriterionResult>, String> {
    let json = json_span(reply).ok_or_else(|| "the reply had no JSON object".to_owned())?;
    let wire: WireReply =
        serde_json::from_str(json).map_err(|e| format!("the reply is not valid: {e}"))?;
    Ok(rubric
        .iter()
        .map(|point| {
            let found = wire
                .criteria_results
                .iter()
                .find(|w| w.criterion_text.trim().eq_ignore_ascii_case(point.trim()));
            match found {
                Some(w) => CriterionResult {
                    criterion: point.clone(),
                    met: w.met,
                    checked: true,
                    feedback: w.feedback.clone(),
                },
                None => CriterionResult {
                    criterion: point.clone(),
                    met: false,
                    checked: true,
                    feedback: "Not assessed by the model.".to_owned(),
                },
            }
        })
        .collect())
}
