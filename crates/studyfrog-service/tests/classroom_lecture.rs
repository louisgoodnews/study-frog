#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0290: the lecture turns with a fake model: teacher segments, student questions, user interjections, refused and failed turns, and what is stored after each.

mod classroom_common;

use classroom_common::*;

use std::cell::RefCell;

use studyfrog_agent::lecture_prompt::PromptMessage;
use studyfrog_core::time::UnixMs;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_turns::{advance_segment, simulated_question, user_message};

#[test]
fn a_multi_turn_lecture_moves_through_the_material_and_every_turn_is_stored() {
    let (_d, s) = vault("lecture");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let first = advance_segment(&lecture, &mut fake(&log), AT).unwrap();
    let second = advance_segment(&lecture, &mut fake(&log), UnixMs(6_000)).unwrap();
    let third = advance_segment(&lecture, &mut fake(&log), UnixMs(7_000)).unwrap();
    assert_eq!(first, "Teacher reply 1.");
    assert_eq!(third, "Teacher reply 3.");

    let prompts = log.borrow();
    assert!(
        prompts[0].contains("Mitochondria"),
        "segment one is the stack's note"
    );
    assert!(
        !prompts[1].contains("Mitochondria"),
        "segment two moves past it"
    );
    assert!(
        prompts[1].contains("cell wall"),
        "segment two is the next note"
    );
    assert!(
        prompts[1].contains(&format!("Dr Lin: {first}")),
        "the reply is in the recent conversation"
    );
    assert_eq!(second, "Teacher reply 2.");

    let turns = stored(&s, id).turns;
    let texts: Vec<&str> = turns.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(texts, [first.as_str(), second.as_str(), third.as_str()]);
    assert!(turns.iter().all(|t| t.speaker == 0));
}

#[test]
fn a_simulated_student_asks_and_the_teacher_answers_and_repeats_are_avoided() {
    let (_d, s) = vault("students");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let (q1, a1) = simulated_question(&lecture, &mut fake(&log), AT, 1).unwrap();
    assert_eq!(q1, "Question 1?");
    assert_eq!(a1, "Teacher answer 2.");
    let (q2, _) = simulated_question(&lecture, &mut fake(&log), AT, 1).unwrap();
    assert_eq!(q2, "Question 3?");
    let (_, _) = simulated_question(&lecture, &mut fake(&log), AT, 2).unwrap();

    let prompts = log.borrow();
    assert!(prompts[0].contains("You are Ada"));
    assert!(
        prompts[2].contains("- Question 1?"),
        "the second question avoids the first"
    );
    assert!(prompts[4].contains("skeptic"), "Ben has the skeptic stance");

    let turns = stored(&s, id).turns;
    let speakers: Vec<usize> = turns.iter().map(|t| t.speaker).collect();
    assert_eq!(speakers, [1, 0, 1, 0, 2, 0]);
}

#[test]
fn a_user_interjection_is_stored_before_the_teacher_answers_it() {
    let (_d, s) = vault("interject");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let mut check = |messages: &[PromptMessage]| -> Result<String, String> {
        let stored_turns = stored(&s, id).turns;
        assert!(
            stored_turns.iter().any(|t| t.text == "What is ATP?"),
            "the user's turn is stored before the model runs"
        );
        assert!(messages[1].content.contains("What is ATP?"));
        Ok("ATP stores energy.".to_owned())
    };
    let answer = user_message(&lecture, &mut check, AT, 3, "  What is ATP?  ").unwrap();
    assert_eq!(answer, "ATP stores energy.");
    let turns = stored(&s, id).turns;
    assert_eq!(turns[0].speaker, 3);
    assert_eq!(turns[0].text, "What is ATP?");
    assert_eq!(turns[1].speaker, 0);
}

#[test]
fn an_interjection_by_someone_else_or_an_empty_one_changes_nothing() {
    let (_d, s) = vault("refused");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    assert!(user_message(&lecture, &mut fake(&log), AT, 1, "hi").is_err());
    assert!(user_message(&lecture, &mut fake(&log), AT, 3, "   ").is_err());
    assert!(
        log.borrow().is_empty(),
        "no model call for a refused message"
    );
    assert!(stored(&s, id).turns.is_empty());
}

#[test]
fn a_failed_model_call_stores_no_teacher_turn() {
    let (_d, s) = vault("failure");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let mut down = |_: &[PromptMessage]| -> Result<String, String> {
        Err("the model is not running".to_owned())
    };
    let err = advance_segment(&lecture, &mut down, AT).unwrap_err();
    assert!(format!("{err:?}").contains("not running"));
    assert!(stored(&s, id).turns.is_empty());
}
