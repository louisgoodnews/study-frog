//! Rubric evaluation of case-study justifications (STORY-0180). The local model answers in JSON,
//! which is parsed here; when no model is running, a deterministic keyword check gives the same
//! report shape. Scoring is always recomputed here, so the model cannot pick its own total.

use serde::Deserialize;

/// Share of the score for choosing correctly; the rest is the rubric.
pub const CHOICE_WEIGHT: f32 = 0.4;
/// Share of the score for the rubric points met.
pub const RUBRIC_WEIGHT: f32 = 0.6;
/// What the keyword check cannot tell the student.
pub const KEYWORD_CAVEAT: &str = "Checked by word overlap, without a model: a correct answer in other words can be marked missing.";

/// Share of a criterion's significant words that must appear for it to count as met.
pub const MATCH_SHARE: f32 = 0.6;

/// Whether one criterion was met, and what to tell the student.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionResult {
    /// The criterion, as written in the rubric.
    pub criterion: String,
    /// Whether the justification makes the point.
    pub met: bool,
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

/// Overall score from the choice and the criteria, 0 to 100.
pub fn overall_pct(choice_valid: bool, criteria: &[CriterionResult]) -> f32 {
    let rubric = if criteria.is_empty() {
        1.0
    } else {
        criteria.iter().filter(|c| c.met).count() as f32 / criteria.len() as f32
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
            let met = !wanted.is_empty() && covered as f32 >= MATCH_SHARE * wanted.len() as f32;
            let feedback = if met {
                "Covered.".to_owned()
            } else {
                let hints: Vec<&str> = missing.iter().take(3).map(|w| w.as_str()).collect();
                format!("Not yet covered. Consider: {}.", hints.join(", "))
            };
            CriterionResult {
                criterion: criterion.clone(),
                met,
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
                    feedback: w.feedback.clone(),
                },
                None => CriterionResult {
                    criterion: criterion.clone(),
                    met: false,
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

/// The instructions for checking one case's answer with the local model. The model works in two stages
/// in its reply: the choice first, then each rubric point against the justification alone. It ends with
/// one JSON object, read by [`parse_model_report`].
pub fn case_system(
    vignette: &str,
    decision: &str,
    options: &[String],
    expected: &[String],
    rubric: &[String],
) -> String {
    let mut out = format!(
        "You check a student's answer to a case study. Work in two stages, then finish.\n\n\
         Stage 1, the choice. The options were: {}. The expected choice is: {}. Say whether the student's \
         choice is the expected one: choice_valid is true only then.\n\n\
         Stage 2, the rubric. For each criterion below, look for the point in the student's justification only, \
         not in the case. Mark it met only when the justification makes that point. In its feedback, quote the \
         words that show it, or say in one sentence what is missing.\n\n\
         Case: {vignette}\nDecision: {decision}\n\nCriteria:\n",
        options.join("; "),
        expected.join("; ")
    );
    for c in rubric {
        out.push_str(&format!("- {c}\n"));
    }
    out.push_str(
        "\nEnd with ONE JSON object on its own, like {\"choice_valid\": true, \"criteria_results\": \
         [{\"criterion_text\": \"<criterion exactly as listed>\", \"met\": true, \"feedback\": \"<one sentence>\"}], \
         \"caveats\": \"<what the student should double-check, or empty>\"}. Give every criterion once.",
    );
    out
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
    fn the_prompt_names_the_choice_the_criteria_and_the_json_shape() {
        let sys = case_system(
            "A clinic screens for anxiety.",
            "Which instrument?",
            &["Short screen".to_owned(), "Interview".to_owned()],
            &["Short screen".to_owned()],
            &["addresses scoring speed".to_owned()],
        );
        for part in [
            "expected choice is: Short screen",
            "- addresses scoring speed",
            "criteria_results",
            "caveats",
        ] {
            assert!(sys.contains(part), "missing {part}: {sys}");
        }
    }

    #[test]
    fn a_model_report_with_caveats_is_read_and_its_caveats_kept() {
        let json = r#"{"choice_valid": true, "criteria_results": [{"criterion_text": "addresses scoring speed", "met": true, "feedback": "Quoted."}], "caveats": "Check the norms."}"#;
        let r = parse_model_report(json, &["addresses scoring speed".to_owned()]).unwrap();
        assert!(r.from_model && r.choice_valid);
        assert_eq!(r.caveats.as_deref(), Some("Check the norms."));
    }
}
