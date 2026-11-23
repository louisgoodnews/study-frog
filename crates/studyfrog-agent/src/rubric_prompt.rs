//! The instructions the local model gets for a case study's check (STORY-0180). Pure text.

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
}
