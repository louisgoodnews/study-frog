#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Goals under a heading of a stack's body.

use proptest::prelude::*;
use studyfrog_md::goals::{goals_of, goals_problem, with_goals};

const BODY: &str =
    "Intro text.\n\n## Goals\n\n- pass the exam\n- [ ] learn 100 verbs\n\n## Notes\n\nKeep this.\n";

#[test]
fn goals_are_read_from_their_section_only() {
    assert_eq!(goals_of(BODY), "- pass the exam\n- [ ] learn 100 verbs");
    assert_eq!(goals_of("no section"), "");
    assert_eq!(
        goals_of("```\n## Goals\n- fenced\n```\n"),
        "",
        "a fenced heading is not a heading"
    );
}

#[test]
fn replacing_keeps_everything_else_and_adding_appends_the_section() {
    let next = with_goals(BODY, "- one");
    assert_eq!(
        next,
        "Intro text.\n\n## Goals\n\n- one\n\n## Notes\n\nKeep this.\n"
    );
    assert_eq!(with_goals("Intro.\n", "- a"), "Intro.\n\n## Goals\n\n- a\n");
    assert_eq!(with_goals("", "- a"), "## Goals\n\n- a\n");
    assert_eq!(
        with_goals(BODY, "  \n"),
        "Intro text.\n\n## Notes\n\nKeep this.\n",
        "empty goals remove the section"
    );
    assert_eq!(with_goals("", ""), "");
}

#[test]
fn a_goal_with_a_heading_line_is_refused() {
    assert!(goals_problem("- fine\n- [x] done").is_none());
    assert!(goals_problem("- a\n# Title").is_some());
    assert!(goals_problem("#hashtag is fine").is_none());
}

proptest! {
    #[test]
    fn goals_survive_a_round_trip_and_the_rest_of_the_body_is_kept(
        goals in "[a-z \\[\\]x-]{1,20}(\n[a-z \\[\\]x-]{1,20}){0,4}",
        rest in "[a-z .]{0,30}"
    ) {
        let body = if rest.trim().is_empty() { String::new() } else { format!("{}\n", rest.trim()) };
        let written = with_goals(&body, &goals);
        prop_assert_eq!(goals_of(&written), goals.trim());
        prop_assert_eq!(with_goals(&written, ""), body);
    }
}
