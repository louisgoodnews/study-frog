#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Teacher task generation (STORY-0210, TASK-0291): the prompt, the JSON parser and the deterministic
//! fallback. No model is called.

use studyfrog_agent::lecture_prompt::{Line, PromptRole};
use studyfrog_agent::lecture_task::{
    MAX_RUBRIC_POINTS, TaskMode, fallback_task, parse_task, task_messages,
};

const SOLO_JSON: &str = r#"{"title": "Cell walls", "prompt": "Say what a cell wall does.", "mode": "solo", "rubric": ["Names the function", " ", "Gives an example"]}"#;

#[test]
fn a_reply_with_json_inside_prose_parses_into_a_solo_task() {
    let reply = format!("Here is the task: {SOLO_JSON} Good luck.");
    let task = parse_task(&reply, TaskMode::Solo).unwrap();
    assert_eq!(task.title, "Cell walls");
    assert_eq!(task.prompt, "Say what a cell wall does.");
    assert_eq!(task.mode, TaskMode::Solo);
    assert_eq!(task.rubric, ["Names the function", "Gives an example"]);
}

#[test]
fn a_group_reply_parses_only_for_group_mode() {
    let json = SOLO_JSON.replace("\"solo\"", "\"group\"");
    assert_eq!(
        parse_task(&json, TaskMode::Group).unwrap().mode,
        TaskMode::Group
    );
    assert!(
        parse_task(&json, TaskMode::Solo).is_err(),
        "a group reply is not a solo task"
    );
    assert!(
        parse_task(SOLO_JSON, TaskMode::Group).is_err(),
        "a solo reply is not a group task"
    );
}

#[test]
fn a_reply_without_a_usable_task_is_refused() {
    assert!(parse_task("no json at all", TaskMode::Solo).is_err());
    assert!(parse_task("{not json}", TaskMode::Solo).is_err());
    let no_prompt = SOLO_JSON.replace("Say what a cell wall does.", "  ");
    assert!(parse_task(&no_prompt, TaskMode::Solo).is_err());
    let no_rubric = SOLO_JSON.replace(r#"["Names the function", " ", "Gives an example"]"#, "[]");
    assert!(parse_task(&no_rubric, TaskMode::Solo).is_err());
    let blank_rubric = SOLO_JSON.replace(
        r#"["Names the function", " ", "Gives an example"]"#,
        r#"[" "]"#,
    );
    assert!(parse_task(&blank_rubric, TaskMode::Solo).is_err());
}

#[test]
fn rubric_points_are_capped() {
    let points: Vec<String> = (0..MAX_RUBRIC_POINTS + 4)
        .map(|i| format!("\"point {i}\""))
        .collect();
    let json = format!(
        r#"{{"title": "T", "prompt": "P", "mode": "solo", "rubric": [{}]}}"#,
        points.join(",")
    );
    let task = parse_task(&json, TaskMode::Solo).unwrap();
    assert_eq!(task.rubric.len(), MAX_RUBRIC_POINTS);
}

#[test]
fn the_fallback_is_deterministic_and_usable_for_either_mode() {
    let solo = fallback_task("Cell walls", TaskMode::Solo);
    assert_eq!(solo, fallback_task("Cell walls", TaskMode::Solo));
    assert!(solo.prompt.contains("Cell walls"));
    assert!(!solo.rubric.is_empty() && solo.rubric.iter().all(|p| !p.trim().is_empty()));
    let group = fallback_task("  ", TaskMode::Group);
    assert_eq!(group.mode, TaskMode::Group);
    assert!(
        group.prompt.contains("this lecture"),
        "a blank topic falls back to a plain phrase"
    );
    assert!(group.prompt.contains("each member"));
}

#[test]
fn the_task_prompt_names_the_mode_and_carries_the_material_and_conversation() {
    let turns = [Line {
        speaker: "Cleo",
        text: "What is a wall?",
    }];
    let messages = task_messages(
        "Dr Lin",
        &["Cell walls give shape."],
        &turns,
        TaskMode::Group,
    );
    assert_eq!(messages[0].role, PromptRole::System);
    assert!(messages[0].content.contains("You are Dr Lin"));
    assert!(messages[0].content.contains("\"mode\" (\"group\")"));
    assert!(messages[1].content.contains("Cell walls give shape."));
    assert!(messages[1].content.contains("Cleo: What is a wall?"));
}
