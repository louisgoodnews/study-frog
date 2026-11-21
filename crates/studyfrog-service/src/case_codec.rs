//! Case studies as question files (STORY-0179). A case is a question whose prompt is the vignette and
//! whose accepted answers are the expected choices, so a reader of plain questions still sees a
//! question with its rubric. The decision and the options are extra front-matter lines. Pure.

use studyfrog_core::items::DEFAULT_MIN_SCORE;
use studyfrog_core::meta::Meta;
use studyfrog_md::doc::{Doc, QuestionDoc};
use studyfrog_md::extra::{extra_value, with_extra};
use studyfrog_rehearsal::case_study::{CaseStudyItem, validate_case_study};

/// Separates the options in the `options` line.
const OPTION_SEP: &str = " | ";
/// Front-matter key of the decision prompt.
const DECISION_KEY: &str = "decision";
/// Front-matter key of the options; its presence marks a question as a case study.
const OPTIONS_KEY: &str = "options";

/// The question document of a case study. The reference answer is a line naming the expected choices.
pub fn case_to_question(c: &CaseStudyItem, meta: Meta) -> Doc {
    let doc = Doc::Question(QuestionDoc {
        meta,
        prompt: c.vignette.trim().to_owned(),
        reference: format!("Expected: {}", c.expected_choices.join(", ")),
        accepted: c.expected_choices.clone(),
        rubric: c.rubric_criteria.clone(),
        min_score: DEFAULT_MIN_SCORE,
        extra: Vec::new(),
    });
    let doc = with_extra(&doc, DECISION_KEY, Some(&single_line(&c.decision_prompt)));
    with_extra(&doc, OPTIONS_KEY, Some(&c.options.join(OPTION_SEP)))
}

/// The value as one line of front matter: line breaks become spaces.
fn single_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The case study a question document holds: `Ok(None)` for a plain question, `Err` for a case study
/// that is not well formed.
pub fn case_from_doc(doc: &Doc) -> Result<Option<CaseStudyItem>, String> {
    let Doc::Question(q) = doc else {
        return Ok(None);
    };
    let Some(options) = extra_value(doc, OPTIONS_KEY) else {
        return Ok(None);
    };
    let case = CaseStudyItem {
        vignette: q.prompt.trim().to_owned(),
        decision_prompt: extra_value(doc, DECISION_KEY).unwrap_or_default(),
        options: options
            .split(OPTION_SEP)
            .map(|o| o.trim().to_owned())
            .collect(),
        expected_choices: q.accepted.clone(),
        rubric_criteria: q.rubric.clone(),
    };
    validate_case_study(&case)?;
    Ok(Some(case))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn sample() -> CaseStudyItem {
        CaseStudyItem {
            vignette: "A clinic wants a short screen for anxiety. Staff are rushed and the test must be quick to score.".to_owned(),
            decision_prompt: "Which instrument should the clinic use?".to_owned(),
            options: vec!["Short screen".to_owned(), "Full interview".to_owned(), "Self-report scale".to_owned()],
            expected_choices: vec!["Short screen".to_owned()],
            rubric_criteria: vec!["addresses scoring speed".to_owned(), "cites criterion validity".to_owned()],
        }
    }

    #[test]
    fn a_case_round_trips_through_its_question_document() {
        let meta = Meta {
            uid: studyfrog_core::uid::Uid(7),
            title: "Screen".to_owned(),
            tags: Vec::new(),
            aliases: Vec::new(),
            created: None,
            updated: None,
        };
        let doc = case_to_question(&sample(), meta);
        assert_eq!(case_from_doc(&doc).unwrap(), Some(sample()));
    }

    #[test]
    fn a_plain_question_is_not_a_case() {
        let meta = Meta {
            uid: studyfrog_core::uid::Uid(8),
            title: "Plain".to_owned(),
            tags: Vec::new(),
            aliases: Vec::new(),
            created: None,
            updated: None,
        };
        let doc = Doc::Question(QuestionDoc {
            meta,
            prompt: "What is validity?".to_owned(),
            reference: "Whether it measures the construct.".to_owned(),
            accepted: Vec::new(),
            rubric: Vec::new(),
            min_score: DEFAULT_MIN_SCORE,
            extra: Vec::new(),
        });
        assert_eq!(case_from_doc(&doc).unwrap(), None);
    }
}
