#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Simulated teammate prompts (STORY-0210, TASK-0291): the persona, the task and the group thread.

use studyfrog_agent::lecture_prompt::{Line, PromptRole};
use studyfrog_agent::lecture_student::Persona;
use studyfrog_agent::lecture_teammate::teammate_messages;

fn ada() -> Persona<'static> {
    Persona {
        name: "Ada",
        stance: "a curious beginner",
    }
}

#[test]
fn the_teammate_sees_the_task_and_the_group_thread() {
    let thread = [
        Line {
            speaker: "Cleo",
            text: "Let's start with the wall.",
        },
        Line {
            speaker: "Ben",
            text: "It gives shape.",
        },
    ];
    let messages = teammate_messages(&ada(), "Walls", "Say what a wall does.", &thread);
    assert_eq!(messages[0].role, PromptRole::System);
    assert!(messages[0].content.contains("You are Ada"));
    assert!(messages[0].content.contains("curious beginner"));
    assert!(
        messages[0]
            .content
            .contains("\"Walls\": Say what a wall does.")
    );
    assert!(
        messages[0]
            .content
            .contains("Do not write the group's final answer")
    );
    assert_eq!(messages[1].role, PromptRole::User);
    assert!(
        messages[1]
            .content
            .ends_with("Cleo: Let's start with the wall.\nBen: It gives shape.")
    );
}

#[test]
fn an_empty_thread_says_so() {
    let messages = teammate_messages(&ada(), "Walls", "Say what a wall does.", &[]);
    assert!(messages[1].content.contains("(none yet)"));
}
