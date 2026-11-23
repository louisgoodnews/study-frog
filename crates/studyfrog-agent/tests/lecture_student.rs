#![allow(clippy::unwrap_used)]
//! Simulated student prompts: persona framing, role isolation and non-repetition (STORY-0209, TASK-0289).

use studyfrog_agent::lecture_prompt::{Line, PromptRole};
use studyfrog_agent::lecture_student::{PRESET_PERSONAS, Persona, student_messages};

#[test]
fn the_three_presets_have_distinct_names_and_stances() {
    let names: std::collections::HashSet<_> = PRESET_PERSONAS.iter().map(|p| p.name).collect();
    let stances: std::collections::HashSet<_> = PRESET_PERSONAS.iter().map(|p| p.stance).collect();
    assert_eq!(names.len(), 3);
    assert_eq!(stances.len(), 3);
}

#[test]
fn a_student_gets_one_system_then_one_user_message_with_the_persona() {
    let persona = Persona {
        name: "Ben",
        stance: "a skeptic who asks for the exact condition",
    };
    let msgs = student_messages(&persona, &["Heaps are trees."], "", &[], &[]);
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].role, PromptRole::System);
    assert_eq!(msgs[1].role, PromptRole::User);
    assert!(msgs[0].content.contains("You are Ben"));
    assert!(
        msgs[0]
            .content
            .contains("a skeptic who asks for the exact condition")
    );
    assert!(msgs[0].content.contains("Ask exactly one question"));
}

#[test]
fn a_student_never_gets_the_teacher_framing_and_each_persona_keeps_its_own() {
    let turns = [Line {
        speaker: "Teacher",
        text: "Heaps are trees.",
    }];
    for persona in PRESET_PERSONAS {
        let sys = &student_messages(&persona, &[], "", &turns, &[])[0].content;
        assert!(sys.contains(persona.name));
        assert!(
            !sys.contains("You are Dr."),
            "teacher framing leaked: {sys}"
        );
        assert!(!sys.contains("Explain only from the lecture material"));
        for other in PRESET_PERSONAS.iter().filter(|o| o.name != persona.name) {
            assert!(
                !sys.contains(other.name),
                "{} leaked into {}",
                other.name,
                persona.name
            );
        }
    }
}

#[test]
fn previous_questions_are_listed_so_the_new_one_does_not_repeat_them() {
    let persona = PRESET_PERSONAS[0];
    let asked = ["What is a heap?", "Why is it a tree?"];
    let user = &student_messages(&persona, &[], "", &[], &asked)[1].content;
    assert!(user.contains("Questions you already asked:\n- What is a heap?\n- Why is it a tree?"));
}

#[test]
fn with_no_earlier_questions_the_list_says_none_yet() {
    let user = &student_messages(&PRESET_PERSONAS[1], &[], "", &[], &[])[1].content;
    assert!(user.ends_with("Questions you already asked:\n(none yet)"));
}
