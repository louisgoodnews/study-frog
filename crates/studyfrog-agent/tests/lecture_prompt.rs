#![allow(clippy::unwrap_used)]
//! Teacher and summary prompt structure and role isolation (STORY-0209, TASK-0289).

use studyfrog_agent::lecture_prompt::{
    Line, PromptRole, TeacherMove, summary_messages, teacher_messages,
};

fn turns() -> Vec<Line<'static>> {
    vec![
        Line {
            speaker: "Ada",
            text: "What is a stack?",
        },
        Line {
            speaker: "Teacher",
            text: "A last-in, first-out list.",
        },
    ]
}

#[test]
fn the_teacher_gets_one_system_then_one_user_message() {
    let msgs = teacher_messages(
        "Dr. Lin",
        &["Stacks push and pop."],
        "",
        &turns(),
        TeacherMove::NextSegment,
    );
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].role, PromptRole::System);
    assert_eq!(msgs[1].role, PromptRole::User);
}

#[test]
fn the_teacher_system_names_the_teacher_and_the_move() {
    for (step, task) in [
        (TeacherMove::NextSegment, "explain the next segment"),
        (TeacherMove::AnswerQuestion, "answer the latest question"),
        (TeacherMove::CheckComprehension, "ask one short question"),
    ] {
        let sys = &teacher_messages("Dr. Lin", &[], "", &[], step)[0].content;
        assert!(sys.contains("You are Dr. Lin"), "{sys}");
        assert!(sys.contains(task), "missing {task}: {sys}");
    }
}

#[test]
fn the_user_message_carries_material_summary_and_turns_in_order() {
    let msgs = teacher_messages(
        "Dr. Lin",
        &["Part one.", "", "Part two."],
        "Stacks were introduced.",
        &turns(),
        TeacherMove::AnswerQuestion,
    );
    let user = &msgs[1].content;
    assert!(
        user.contains("Part one.\n---\nPart two."),
        "empty excerpts are dropped: {user}"
    );
    assert!(user.contains("Summary so far:\nStacks were introduced."));
    let ask = user.find("Ada: What is a stack?").unwrap();
    let answer = user.find("Teacher: A last-in, first-out list.").unwrap();
    assert!(ask < answer, "turns keep their order");
}

#[test]
fn an_empty_lecture_and_no_history_say_so_instead_of_inventing_content() {
    let msgs = teacher_messages("Dr. Lin", &[], "  ", &[], TeacherMove::NextSegment);
    let user = &msgs[1].content;
    assert!(user.starts_with("Lecture material:\n(no material)"));
    assert!(user.contains("Summary so far:\n(none yet)"));
    assert!(user.ends_with("Recent conversation:\n(none yet)"));
}

#[test]
fn the_teacher_system_never_carries_the_student_framing() {
    let sys = &teacher_messages("Dr. Lin", &[], "", &[], TeacherMove::NextSegment)[0].content;
    assert!(!sys.contains("You are Ada"));
    assert!(!sys.contains("a student in a study classroom"));
    assert!(!sys.contains("Ask exactly one question"));
}

#[test]
fn the_summary_prompt_states_the_word_limit_and_folds_in_the_previous_summary() {
    let msgs = summary_messages(
        &["Stacks push and pop."],
        "Lists were covered.",
        &turns(),
        80,
    );
    assert_eq!(msgs[0].role, PromptRole::System);
    assert!(msgs[0].content.contains("at most 80 words"));
    assert!(
        msgs[1]
            .content
            .contains("Previous summary:\nLists were covered.")
    );
    assert!(
        msgs[1]
            .content
            .contains("New lines:\nAda: What is a stack?")
    );
}

#[test]
fn the_summary_prompt_with_nothing_to_fold_says_none_yet() {
    let msgs = summary_messages(&[], "", &[], 40);
    assert!(msgs[1].content.contains("Previous summary:\n(none yet)"));
    assert!(msgs[1].content.contains("(no material)"));
    assert!(msgs[1].content.ends_with("New lines:\n(none yet)"));
}
