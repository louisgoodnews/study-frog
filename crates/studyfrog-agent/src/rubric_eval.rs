//! Rubric evaluation of case-study justifications (STORY-0180). The local model answers in JSON,
//! which is parsed here; when no model is running, a deterministic keyword check gives the same
//! report shape. Scoring is always recomputed here, so the model cannot pick its own total.

use serde::Deserialize;

pub use crate::rubric_prompt::case_system;

/// Share of the score for choosing correctly; the rest is the rubric.
pub const CHOICE_WEIGHT: f32 = 0.4;
/// Share of the score for the rubric points met.
pub const RUBRIC_WEIGHT: f32 = 0.6;
/// What the keyword check cannot tell the student.
pub const KEYWORD_CAVEAT: &str = "Checked by word overlap, without a model: a correct answer in other words can be marked missing.";

/// Share of a criterion's significant words that must appear for it to count as met.
pub const MATCH_SHARE: f32 = 0.6;
/// What the student is told about a point the keyword check cannot test.
pub const NOT_CHECKED: &str =
    "Not checked by word overlap: this point has no word of four letters or more to match.";

/// Whether one criterion was met, and what to tell the student.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionResult {
    /// The criterion, as written in the rubric.
    pub criterion: String,
    /// Whether the justification makes the point.
    pub met: bool,
    /// Whether the point was tested at all. The keyword check cannot test a point with no word of four
    /// letters or more; such a point is not checked, and it is neither met nor missing.
    pub checked: bool,
    /// One sentence for the student.
    pub feedback: String,
}

/// The evaluation of one answer.
#[derive(Debug, Clone, PartialEq)]
pub struct EvalReport {
    /// Whether the choice matches the expected one.
    pub choice_valid: bool,
    /// One result per rubric criterion, in rubric order.
    pub criteria: Vec<CriterionResult>,
    /// Overall score, 0 to 100.
    pub overall_pct: f32,
    /// True when the local model produced the judgements.
    pub from_model: bool,
    /// Anything the student should double-check (from the model, or the limits of the keyword check).
    pub caveats: Option<String>,
}

/// Overall score from the choice and the criteria, 0 to 100. Criteria that were not checked do not count.
pub fn overall_pct(choice_valid: bool, criteria: &[CriterionResult]) -> f32 {
    let checked: Vec<&CriterionResult> = criteria.iter().filter(|c| c.checked).collect();
    let rubric = if checked.is_empty() {
        1.0
    } else {
        checked.iter().filter(|c| c.met).count() as f32 / checked.len() as f32
    };
    let choice = if choice_valid { 1.0 } else { 0.0 };
    ((CHOICE_WEIGHT * choice + RUBRIC_WEIGHT * rubric) * 100.0).clamp(0.0, 100.0)
}

const STOPWORDS: [&str; 16] = [
    "that", "this", "with", "from", "their", "there", "which", "about", "would", "should", "could",
    "into", "have", "been", "they", "such",
];

/// Significant words of a text: lower case, at least four letters, no stopwords, no repeats.
fn significant_words(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for w in text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 4 && !STOPWORDS.contains(w))
    {
        if !out.iter().any(|x| x == w) {
            out.push(w.to_owned());
        }
    }
    out
}

/// Deterministic check: a criterion is met when enough of its significant words appear in the
/// justification. Used when no model is running.
pub fn fallback_evaluate(justification: &str, choice_valid: bool, rubric: &[String]) -> EvalReport {
    let have = significant_words(justification);
    let criteria: Vec<CriterionResult> = rubric
        .iter()
        .map(|criterion| {
            let wanted = significant_words(criterion);
            let missing: Vec<&String> = wanted.iter().filter(|w| !have.contains(w)).collect();
            let covered = wanted.len() - missing.len();
            let checked = !wanted.is_empty();
            let met = checked && covered as f32 >= MATCH_SHARE * wanted.len() as f32;
            let feedback = if !checked {
                NOT_CHECKED.to_owned()
            } else if met {
                "Covered.".to_owned()
            } else {
                let hints: Vec<&str> = missing.iter().take(3).map(|w| w.as_str()).collect();
                format!("Not yet covered. Consider: {}.", hints.join(", "))
            };
            CriterionResult {
                criterion: criterion.clone(),
                met,
                checked,
                feedback,
            }
        })
        .collect();
    EvalReport {
        choice_valid,
        overall_pct: overall_pct(choice_valid, &criteria),
        criteria,
        from_model: false,
        caveats: Some(KEYWORD_CAVEAT.to_owned()),
    }
}

#[derive(Deserialize)]
struct WireReport {
    choice_valid: bool,
    criteria_results: Vec<WireCriterion>,
    #[serde(default)]
    caveats: Option<String>,
}

#[derive(Deserialize)]
struct WireCriterion {
    criterion_text: String,
    met: bool,
    #[serde(default)]
    feedback: String,
}

/// Parse the model's JSON (`choice_valid` and `criteria_results`, each with `criterion_text`,
/// `met` and `feedback`). Results are matched to the rubric by text, ignoring case; a criterion
/// the model did not mention is reported as not met.
pub fn parse_model_report(json: &str, rubric: &[String]) -> Result<EvalReport, String> {
    let wire: WireReport =
        serde_json::from_str(json).map_err(|e| format!("the model's answer is not valid: {e}"))?;
    let criteria: Vec<CriterionResult> = rubric
        .iter()
        .map(|criterion| {
            let found = wire.criteria_results.iter().find(|w| {
                w.criterion_text
                    .trim()
                    .eq_ignore_ascii_case(criterion.trim())
            });
            match found {
                Some(w) => CriterionResult {
                    criterion: criterion.clone(),
                    met: w.met,
                    checked: true,
                    feedback: w.feedback.clone(),
                },
                None => CriterionResult {
                    criterion: criterion.clone(),
                    met: false,
                    checked: true,
                    feedback: "Not assessed by the model.".to_owned(),
                },
            }
        })
        .collect();
    Ok(EvalReport {
        choice_valid: wire.choice_valid,
        overall_pct: overall_pct(wire.choice_valid, &criteria),
        criteria,
        from_model: true,
        caveats: wire.caveats.filter(|c| !c.trim().is_empty()),
    })
}

/// The part of a model reply that is the JSON object: from its first `{` to its last `}`.
pub fn json_span(reply: &str) -> Option<&str> {
    let start = reply.find('{')?;
    let end = reply.rfind('}')?;
    (start < end).then(|| &reply[start..=end])
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_json_is_found_inside_prose() {
        let reply = "Stage one: the choice matches. {\"choice_valid\": true, \"criteria_results\": []} Done.";
        assert_eq!(
            json_span(reply),
            Some("{\"choice_valid\": true, \"criteria_results\": []}")
        );
        assert_eq!(json_span("no json here"), None);
    }

    #[test]
    fn a_model_report_with_caveats_is_read_and_its_caveats_kept() {
        let json = r#"{"choice_valid": true, "criteria_results": [{"criterion_text": "addresses scoring speed", "met": true, "feedback": "Quoted."}], "caveats": "Check the norms."}"#;
        let r = parse_model_report(json, &["addresses scoring speed".to_owned()]).unwrap();
        assert!(r.from_model && r.choice_valid);
        assert_eq!(r.caveats.as_deref(), Some("Check the norms."));
    }
}
