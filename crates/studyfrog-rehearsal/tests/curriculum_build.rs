//! Building a mock exam from curriculum chunks (STORY-0186, STORY-0207): format plan, grounding filter,
//! offline templates, and assembly.
#![allow(clippy::unwrap_used)]

use studyfrog_rehearsal::curriculum::{Chunk, chunk_markdown};
use studyfrog_rehearsal::curriculum_build::{
    ExamItemFormat, FormatPlan, GeneratedQuestion, accept, assemble, plan, templates,
};
use studyfrog_rehearsal::exam_format::set_plan_count;

fn notes() -> Vec<Chunk> {
    chunk_markdown(
        "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\nValidität: the degree to which a test measures what it claims to measure.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\ns_e = s_x * sqrt(1 - r_xx)\n",
    )
}

fn gen_open(topic: &str, prompt: &str, sample: &str) -> GeneratedQuestion {
    GeneratedQuestion {
        format: ExamItemFormat::OpenQuestion,
        prompt: prompt.to_owned(),
        sample_answer: sample.to_owned(),
        rubric: vec!["names consistency".to_owned()],
        points: 2,
        topic: topic.to_owned(),
        options: Vec::new(),
        correct_indices: Vec::new(),
        gap_hints: Vec::new(),
        gap_answers: Vec::new(),
    }
}

#[test]
fn the_plan_is_forty_thirty_thirty() {
    let mut expected = FormatPlan::default();
    set_plan_count(&mut expected, ExamItemFormat::OpenQuestion, 4);
    set_plan_count(&mut expected, ExamItemFormat::CaseScenario, 3);
    set_plan_count(&mut expected, ExamItemFormat::Calculation, 3);
    assert_eq!(plan(10), expected);
    assert_eq!(plan(0), FormatPlan::default());
}

#[test]
fn a_question_the_notes_support_is_accepted() {
    let q = gen_open(
        "Reliabilität",
        "What is reliability in terms of consistency?",
        "Consistency of measurement over repeated testing occasions.",
    );
    assert_eq!(accept(&q, &notes()), Ok(()));
}

#[test]
fn an_out_of_syllabus_topic_and_an_invented_answer_are_rejected() {
    let mut q = gen_open(
        "Projektive Verfahren",
        "What is the Rorschach test used for?",
        "Projective personality assessment with inkblots and free association.",
    );
    assert!(
        accept(&q, &notes())
            .unwrap_err()
            .contains("out of syllabus")
    );
    q.topic = "Reliabilität".to_owned();
    assert!(accept(&q, &notes()).unwrap_err().contains("hallucination"));
}

#[test]
fn select_and_gap_questions_validate_format_invariants() {
    let bad_single = GeneratedQuestion {
        format: ExamItemFormat::SingleSelect,
        options: vec!["Consistency of measurement".to_owned()],
        correct_indices: vec![0],
        ..gen_open(
            "Reliabilität",
            "Pick the best definition of reliability.",
            "Consistency of measurement",
        )
    };
    assert!(accept(&bad_single, &notes()).is_err());

    let bad_cloze = GeneratedQuestion {
        format: ExamItemFormat::FillInGaps,
        gap_hints: vec![None],
        gap_answers: vec!["consistency".to_owned()],
        ..gen_open(
            "Reliabilität",
            "Reliabilität is {{c1::consistency}} of measurement.",
            "consistency",
        )
    };
    assert!(accept(&bad_cloze, &notes()).is_err());

    let good_cloze = GeneratedQuestion {
        format: ExamItemFormat::FillInGaps,
        gap_hints: vec![None],
        gap_answers: vec!["consistency".to_owned()],
        ..gen_open(
            "Reliabilität",
            "Reliabilität means [1] of measurement.",
            "consistency of measurement",
        )
    };
    assert_eq!(accept(&good_cloze, &notes()), Ok(()));
}

#[test]
fn templates_and_assembly_support_all_six_formats() {
    let mut want = FormatPlan::default();
    for fmt in studyfrog_rehearsal::exam_format::ALL_EXAM_FORMATS {
        set_plan_count(&mut want, fmt, 1);
    }
    let qs = templates(&notes(), want);
    assert_eq!(qs.len(), 6, "{qs:?}");
    let (exam, keys) = assemble("Klausur", &qs);
    assert_eq!(exam.sections.len(), 6);
    assert_eq!(keys.len(), 6);
    assert!(studyfrog_rehearsal::mock_exam::validate_mock_exam(&exam).is_ok());
}

/// BUG-0038: a recall rubric point is the meaning from the notes, so the keyword check can test it.
#[test]
fn a_recall_rubric_point_is_the_meaning_from_the_notes() {
    let chunks = chunk_markdown("# Gueltigkeit\nGueltigkeit: misst, was er soll.\n");
    let qs = templates(&chunks, plan(3));
    let recall = qs
        .iter()
        .find(|q| q.format == ExamItemFormat::OpenQuestion)
        .unwrap();
    assert_eq!(recall.rubric, ["misst, was er soll"]);
    assert_eq!(recall.sample_answer, "Gueltigkeit: misst, was er soll.");
}

/// BUG-0038 and BUG-0053: a calculation's rubric names the quantity and expression without leaking the formula.
#[test]
fn a_calculation_is_asked_for_by_its_quantity_and_checked_by_its_expression() {
    let chunks = chunk_markdown("# Formel\nStandardfehler = s / Wurzel(n) = 2 / 2 = 1\n");
    let qs = templates(&chunks, plan(10));
    let calc = qs
        .iter()
        .find(|q| q.format == ExamItemFormat::Calculation)
        .unwrap();
    assert_eq!(calc.rubric, ["Standardfehler", "s / Wurzel(n)"]);
    assert_eq!(
        calc.prompt,
        "Give the formula for Standardfehler and explain its symbols."
    );
    assert!(!calc.prompt.contains("Wurzel"));
}

/// BUG-0052: a case template's rubric is a sentence of the notes, never the question.
#[test]
fn a_case_rubric_point_is_a_sentence_of_the_notes() {
    let chunks = chunk_markdown("# Mittelwert\nSumme durch Anzahl\n");
    let qs = templates(&chunks, plan(10));
    let case = qs
        .iter()
        .find(|q| q.format == ExamItemFormat::CaseScenario)
        .unwrap();
    assert_eq!(case.rubric, ["Summe durch Anzahl"]);
}
