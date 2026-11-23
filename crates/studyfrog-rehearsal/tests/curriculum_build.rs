//! Building a mock exam from curriculum chunks (STORY-0186): the plan, the filter, the templates and the
//! assembly, tested through their public functions.
#![allow(clippy::unwrap_used)]

use studyfrog_rehearsal::curriculum::{Chunk, chunk_markdown};
use studyfrog_rehearsal::curriculum_build::{
    GeneratedQuestion, Kind, Plan, accept, assemble, plan, templates,
};

fn notes() -> Vec<Chunk> {
    chunk_markdown(
        "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\ns_e = s_x * sqrt(1 - r_xx)\n",
    )
}

#[test]
fn the_plan_is_forty_thirty_thirty() {
    assert_eq!(
        plan(10),
        Plan {
            recall: 4,
            case: 3,
            calc: 3
        }
    );
    let p = plan(3);
    assert!(p.recall >= 1 && p.case >= 1 && p.calc >= 1);
    assert_eq!(
        plan(0),
        Plan {
            recall: 0,
            case: 0,
            calc: 0
        }
    );
}

#[test]
fn a_question_the_notes_support_is_accepted() {
    let q = GeneratedQuestion {
        kind: Kind::Recall,
        prompt: "What is reliability in terms of consistency?".to_owned(),
        sample_answer: "Consistency of measurement over repeated testing occasions.".to_owned(),
        rubric: vec!["names consistency".to_owned()],
        points: 2,
        topic: "Reliabilität".to_owned(),
    };
    assert_eq!(accept(&q, &notes()), Ok(()));
}

#[test]
fn an_out_of_syllabus_topic_and_an_invented_answer_are_rejected() {
    let mut q = GeneratedQuestion {
        kind: Kind::Recall,
        prompt: "What is the Rorschach test used for?".to_owned(),
        sample_answer: "Projective personality assessment with inkblots and free association."
            .to_owned(),
        rubric: vec![],
        points: 2,
        topic: "Projektive Verfahren".to_owned(),
    };
    assert!(
        accept(&q, &notes())
            .unwrap_err()
            .contains("out of syllabus")
    );
    q.topic = "Reliabilität".to_owned();
    assert!(accept(&q, &notes()).unwrap_err().contains("hallucination"));
}

#[test]
fn templates_follow_the_plan_and_say_where_their_answers_come_from() {
    let t = templates(
        &notes(),
        Plan {
            recall: 1,
            case: 1,
            calc: 1,
        },
    );
    assert_eq!(t.len(), 3);
    assert!(
        t.iter()
            .any(|q| q.kind == Kind::Calculation && q.sample_answer.contains("sqrt"))
    );
}

#[test]
fn assembly_keeps_the_kinds_in_order_with_their_keys() {
    let qs = templates(
        &notes(),
        Plan {
            recall: 1,
            case: 1,
            calc: 1,
        },
    );
    let (exam, keys) = assemble("Klausur", &qs);
    assert_eq!(exam.sections.len(), 3);
    assert_eq!(keys.len(), 3);
    assert!(studyfrog_rehearsal::mock_exam::validate_mock_exam(&exam).is_ok());
}

/// BUG-0038: a recall rubric point is the meaning from the notes, so the keyword check can test it.
#[test]
fn a_recall_rubric_point_is_the_meaning_from_the_notes() {
    let chunks = chunk_markdown("# Gueltigkeit\nGueltigkeit: misst, was er soll.\n");
    let qs = templates(&chunks, plan(3));
    let recall = qs.iter().find(|q| q.kind == Kind::Recall).unwrap();
    assert_eq!(recall.rubric, ["misst, was er soll"]);
    assert_eq!(recall.sample_answer, "Gueltigkeit: misst, was er soll.");
    assert!(
        !recall
            .rubric
            .iter()
            .any(|r| r.contains("not just the term"))
    );
}

/// BUG-0038 and BUG-0053: a calculation's rubric names the quantity and the expression, and its prompt does not
/// print the formula.
#[test]
fn a_calculation_is_asked_for_by_its_quantity_and_checked_by_its_expression() {
    let chunks = chunk_markdown("# Formel\nStandardfehler = s / Wurzel(n) = 2 / 2 = 1\n");
    let qs = templates(&chunks, plan(10));
    let calc = qs.iter().find(|q| q.kind == Kind::Calculation).unwrap();
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
    let case = qs.iter().find(|q| q.kind == Kind::Case).unwrap();
    assert_eq!(case.rubric, ["Summe durch Anzahl"]);
}
