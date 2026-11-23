#![allow(clippy::unwrap_used)]
//! Context budgeting: caps, truncation and edge cases (STORY-0209, TASK-0289).

use studyfrog_agent::lecture_budget::{chars_for_tokens, line_chars, pack};
use studyfrog_agent::lecture_prompt::Line;

fn line<'a>(speaker: &'a str, text: &'a str) -> Line<'a> {
    Line { speaker, text }
}

#[test]
fn a_small_lecture_and_history_fit_whole() {
    let turns = [line("Ada", "hi"), line("Teacher", "hello")];
    let p = pack(1000, "Short summary.", &["Stacks."], &turns);
    assert_eq!(p.summary, "Short summary.");
    assert_eq!(p.excerpts, vec!["Stacks."]);
    assert_eq!(p.turns, turns.to_vec());
    assert!(p.used_chars <= 1000);
}

#[test]
fn an_empty_lecture_packs_only_the_summary_and_the_turns() {
    let turns = [line("Ada", "What is a heap?")];
    let p = pack(200, "Summary.", &[], &turns);
    assert!(p.excerpts.is_empty());
    assert_eq!(p.summary, "Summary.");
    assert_eq!(p.turns.len(), 1);
}

#[test]
fn an_empty_everything_packs_to_nothing() {
    let p = pack(100, "", &[], &[]);
    assert_eq!(p.used_chars, 0);
    assert!(p.summary.is_empty());
    assert!(p.excerpts.is_empty());
    assert!(p.turns.is_empty());
}

#[test]
fn a_zero_budget_packs_nothing_even_with_material() {
    let turns = [line("Ada", "hi")];
    let p = pack(0, "Summary.", &["Lecture text."], &turns);
    assert_eq!(p.used_chars, 0);
    assert!(p.summary.is_empty());
    assert!(p.excerpts.is_empty());
    assert!(p.turns.is_empty());
}

#[test]
fn a_lecture_larger_than_the_budget_is_cut_short_and_never_overflows() {
    let lecture = "x".repeat(500);
    let p = pack(100, "", &[&lecture], &[]);
    assert_eq!(p.excerpts.len(), 1);
    assert_eq!(p.excerpts[0].chars().count(), 100);
    assert_eq!(p.used_chars, 100);
}

#[test]
fn excerpts_after_the_cut_one_are_dropped() {
    let a = "a".repeat(30);
    let b = "b".repeat(30);
    let c = "c".repeat(30);
    let p = pack(45, "", &[&a, &b, &c], &[]);
    assert_eq!(p.excerpts, vec![a.as_str(), &b[..15]]);
    assert!(p.used_chars <= 45);
}

#[test]
fn the_summary_is_capped_at_a_quarter_of_the_budget() {
    let summary = "s".repeat(400);
    let p = pack(200, &summary, &[], &[]);
    assert_eq!(p.summary.chars().count(), 50);
}

#[test]
fn the_summary_is_cut_on_a_character_boundary() {
    let p = pack(8, "ééééééé", &[], &[]);
    assert_eq!(p.summary, "éé");
}

#[test]
fn recent_turns_keep_the_newest_contiguous_tail_within_half_the_budget() {
    let texts: Vec<String> = (0..10).map(|i| format!("turn {i:02}")).collect();
    let turns: Vec<Line<'_>> = texts.iter().map(|t| line("Ada", t)).collect();
    let budget = 200;
    let p = pack(budget, "", &[], &turns);
    let turn_chars: usize = p.turns.iter().map(line_chars).sum();
    assert!(turn_chars <= budget / 2, "turns used {turn_chars}");
    assert!(!p.turns.is_empty());
    assert_eq!(
        p.turns.last().unwrap().text,
        "turn 09",
        "the newest turn is kept"
    );
    let first = turns.len() - p.turns.len();
    assert_eq!(
        p.turns,
        turns[first..].to_vec(),
        "kept turns are a contiguous tail in order"
    );
}

#[test]
fn packing_stops_at_the_first_turn_that_does_not_fit() {
    let huge = "h".repeat(300);
    let turns = [
        line("Ada", "small"),
        line("Ben", &huge),
        line("Mia", "newest"),
    ];
    let p = pack(100, "", &[], &turns);
    assert_eq!(
        p.turns,
        vec![line("Mia", "newest")],
        "an older turn is not skipped over"
    );
}

#[test]
fn excerpts_get_the_room_the_summary_and_turns_leave() {
    let turns = [line("Ada", "0123456789")];
    let lecture = "l".repeat(500);
    let budget = 100;
    let p = pack(budget, "", &[&lecture], &turns);
    assert_eq!(p.used_chars, budget);
    assert_eq!(
        p.excerpts[0].chars().count(),
        budget - line_chars(&turns[0])
    );
}

#[test]
fn a_token_limit_converts_conservatively_and_saturates() {
    assert_eq!(chars_for_tokens(0), 0);
    assert_eq!(chars_for_tokens(1500), 4500);
    assert_eq!(chars_for_tokens(usize::MAX), usize::MAX);
}
