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
