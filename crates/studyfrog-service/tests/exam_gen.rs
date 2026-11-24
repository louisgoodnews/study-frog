#![allow(clippy::unwrap_used)]
//! Tests for format-aware exam generation and grounding (STORY-0186, STORY-0207).

use studyfrog_rehearsal::curriculum_build::ExamItemFormat;
use studyfrog_rehearsal::exam_format::{
    MixMode, default_format_mix, format_index, section_heading,
};
use studyfrog_rehearsal::mock_exam::validate_mock_exam;
use studyfrog_service::exam_gen::{chunks_of_markdown, generate, generate_with_mix};

const NOTES: &str = "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\nValidität: the degree to which a test measures what it claims to measure.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\n# Normierung\nStandard scores place a result against the norm group mean and standard deviation.\n";

fn json(questions: &str) -> String {
    format!("{{\"questions\": [{questions}]}}")
}

#[test]
fn offline_the_exam_comes_from_templates_and_is_valid() {
    let g = generate(&chunks_of_markdown(NOTES), 6, "Klausur", None);
    assert_eq!(g.from_model, 0);
    assert!(g.from_templates > 0);
    assert!(validate_mock_exam(&g.exam).is_ok());
    assert_eq!(
        g.keys.len(),
        g.exam
            .sections
            .iter()
            .map(|s| s.questions.len())
            .sum::<usize>()
    );
}

#[test]
fn the_model_questions_that_the_notes_support_are_kept_and_the_rest_rejected() {
    let good = r#"{"kind": "recall", "prompt": "What does the reliability coefficient compare?", "sample_answer": "True score variance with observed score variance.", "rubric": ["names both variances"], "points": 2}"#;
    let invented = r#"{"kind": "recall", "prompt": "Describe the Rorschach inkblot method in detail.", "sample_answer": "Projective inkblots reveal unconscious conflicts through free association.", "rubric": [], "points": 2}"#;
    let reply = json(&format!("{good}, {invented}"));
    let mut ask = |_: &str, _: &str| Ok(reply.clone());
    let g = generate(&chunks_of_markdown(NOTES), 3, "Klausur", Some(&mut ask));
    assert_eq!(g.from_model, 1, "{:?}", g.rejected);
    assert!(g.rejected.iter().any(|r| r.contains("hallucination")));
}

#[test]
fn a_failed_chunk_is_noted_and_templates_stand_in() {
    let mut ask = |_: &str, _: &str| Err("the model is not running".to_owned());
    let g = generate(&chunks_of_markdown(NOTES), 3, "Klausur", Some(&mut ask));
    assert_eq!(g.from_model, 0);
    assert!(g.rejected.iter().any(|r| r.contains("not running")));
    assert!(validate_mock_exam(&g.exam).is_ok());
}

/// BUG-0055: a part the notes cannot fill is named in `short`, and the exam has what the notes hold.
#[test]
fn a_part_the_notes_cannot_fill_is_named_and_the_rest_follows_the_plan() {
    let notes = "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\nValidität: the degree to which a test measures what it claims to measure.\ns_e = s_x * sqrt(1 - r_xx)\n";
    let g = generate(&chunks_of_markdown(notes), 10, "Klausur", None);
    assert_eq!(g.keys.len(), 4);
    assert_eq!(g.short.len(), 3);
    assert!(
        g.short
            .iter()
            .any(|s| s.format == ExamItemFormat::CaseScenario && s.made == 1 && s.wanted == 3)
    );
    assert!(
        g.short
            .iter()
            .any(|s| s.format == ExamItemFormat::Calculation && s.made == 1)
    );
    assert!(validate_mock_exam(&g.exam).is_ok());
}

#[test]
fn generate_with_mix_respects_selected_formats_offline() {
    let mut mix = default_format_mix();
    mix.mode = MixMode::Counts;
    for a in &mut mix.formats {
        a.enabled = false;
        a.value = 0;
    }
    for fmt in [
        ExamItemFormat::FillInGaps,
        ExamItemFormat::SingleSelect,
        ExamItemFormat::MultipleSelect,
    ] {
        mix.formats[format_index(fmt)].enabled = true;
        mix.formats[format_index(fmt)].value = 1;
    }
    let g = generate_with_mix(&chunks_of_markdown(NOTES), &mix, "Format mix", None).unwrap();
    assert_eq!(g.keys.len(), 3);
    let headings: Vec<&str> = g.exam.sections.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        headings,
        vec![
            section_heading(ExamItemFormat::FillInGaps),
            section_heading(ExamItemFormat::SingleSelect),
            section_heading(ExamItemFormat::MultipleSelect),
        ]
    );
}

#[test]
fn generate_with_mix_rejects_empty_format_selection() {
    let mut mix = default_format_mix();
    for a in &mut mix.formats {
        a.enabled = false;
    }
    assert!(generate_with_mix(&chunks_of_markdown(NOTES), &mix, "Bad", None).is_err());
}

#[test]
fn generate_with_mix_normalizes_cloze_spans_from_model_reply() {
    let mut mix = default_format_mix();
    mix.mode = MixMode::Counts;
    for a in &mut mix.formats {
        a.enabled = false;
        a.value = 0;
    }
    mix.formats[format_index(ExamItemFormat::FillInGaps)].enabled = true;
    mix.formats[format_index(ExamItemFormat::FillInGaps)].value = 1;

    let reply = json(
        r#"{"kind": "fill_in_gaps", "prompt": "Reliabilität means {{c1::consistency::noun}} over repeated testing.", "sample_answer": "consistency of measurement over repeated testing occasions", "points": 2}"#,
    );
    let mut ask = |_: &str, _: &str| Ok(reply.clone());
    let g = generate_with_mix(&chunks_of_markdown(NOTES), &mix, "Cloze", Some(&mut ask)).unwrap();
    assert_eq!(g.from_model, 1, "{:?}", g.rejected);
    let q = &g.exam.sections[0].questions[0];
    assert!(!q.prompt.contains("{{c1"), "{}", q.prompt);
    assert!(q.prompt.contains("[1]"), "{}", q.prompt);
    assert_eq!(q.gap_hints, vec![Some("noun".to_owned())]);
    assert_eq!(g.keys[0].gap_answers, vec!["consistency".to_owned()]);
}

#[test]
fn generate_with_mix_accepts_grounded_select_items_from_model() {
    let mut mix = default_format_mix();
    mix.mode = MixMode::Counts;
    for a in &mut mix.formats {
        a.enabled = false;
        a.value = 0;
    }
    mix.formats[format_index(ExamItemFormat::SingleSelect)].enabled = true;
    mix.formats[format_index(ExamItemFormat::SingleSelect)].value = 1;

    let reply = json(
        r#"{"kind": "single_select", "prompt": "Which defines reliability?", "options": ["Consistency of measurement over repeated testing occasions", "Unrelated noise"], "correct_index": 0, "points": 2}"#,
    );
    let mut ask = |_: &str, _: &str| Ok(reply.clone());
    let g = generate_with_mix(&chunks_of_markdown(NOTES), &mix, "MCQ", Some(&mut ask)).unwrap();
    assert_eq!(g.from_model, 1, "{:?}", g.rejected);
    assert_eq!(g.keys[0].correct_indices, vec![0]);
    assert_eq!(g.exam.sections[0].questions[0].options.len(), 2);
}

#[test]
fn generate_with_mix_rejects_mismatched_fill_in_gaps_blank_counts() {
    let mut mix = default_format_mix();
    mix.mode = MixMode::Counts;
    for a in &mut mix.formats {
        a.enabled = false;
        a.value = 0;
    }
    mix.formats[format_index(ExamItemFormat::FillInGaps)].enabled = true;
    mix.formats[format_index(ExamItemFormat::FillInGaps)].value = 1;

    let reply = json(
        r#"{"kind": "fill_in_gaps", "prompt": "Reliabilität means [1] over [2] testing.", "gaps": ["consistency"], "sample_answer": "consistency of measurement over repeated testing occasions", "points": 2}, {"kind": "fill_in_gaps", "prompt": "Reliabilität means [foo] over repeated testing.", "gaps": ["consistency"], "sample_answer": "consistency of measurement over repeated testing occasions", "points": 2}"#,
    );
    let mut ask = |_: &str, _: &str| Ok(reply.clone());
    let g =
        generate_with_mix(&chunks_of_markdown(NOTES), &mix, "Bad gaps", Some(&mut ask)).unwrap();
    assert_eq!(g.from_model, 0);
    assert!(g.rejected.iter().any(|r| r.contains("[1]..[n]")));
}

#[test]
fn generate_with_mix_counts_only_kept_model_questions_in_from_model() {
    let mut mix = default_format_mix();
    mix.mode = MixMode::Counts;
    for a in &mut mix.formats {
        a.enabled = false;
        a.value = 0;
    }
    mix.formats[format_index(ExamItemFormat::OpenQuestion)].enabled = true;
    mix.formats[format_index(ExamItemFormat::OpenQuestion)].value = 1;
    mix.formats[format_index(ExamItemFormat::FillInGaps)].enabled = true;
    mix.formats[format_index(ExamItemFormat::FillInGaps)].value = 1;

    let q1 = r#"{"kind": "open_question", "prompt": "What does the reliability coefficient compare?", "sample_answer": "True score variance with observed score variance.", "rubric": ["variances"], "points": 2}"#;
    let q2 = r#"{"kind": "open_question", "prompt": "What is Reliabilität over repeated testing?", "sample_answer": "Consistency of measurement over repeated testing occasions.", "rubric": ["consistency"], "points": 2}"#;
    let q3 = r#"{"kind": "open_question", "prompt": "What does Validität measure in testing?", "sample_answer": "The degree to which a test measures what it claims to measure.", "rubric": ["validity"], "points": 2}"#;
    let reply = json(&format!("{q1}, {q2}, {q3}"));
    let mut ask = |_: &str, _: &str| Ok(reply.clone());
    let g = generate_with_mix(&chunks_of_markdown(NOTES), &mix, "Trimmed", Some(&mut ask)).unwrap();

    assert_eq!(g.keys.len(), 2);
    assert_eq!(g.from_model, 1);
    assert_eq!(g.from_templates, 1);
    assert_eq!(g.from_model + g.from_templates, g.keys.len());
}
