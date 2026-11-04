#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0219: the item page and the library tell what each stack makes of an item.

use studyfrog_core::uid::Uid;
use studyfrog_service::fam_refs::{RefView, per_stack};

fn view(title: &str, primary: bool, percent: u8, difficulty: f32, due_days: i64) -> RefView {
    RefView {
        stack: Uid(1),
        folder: None,
        title: title.into(),
        primary,
        value: Some(f32::from(percent) / 100.0),
        percent: Some(percent),
        difficulty: Some(difficulty),
        stability: None,
        due_ms: Some(due_days * 86_400_000),
        reviews: 3,
    }
}

#[test]
fn one_line_for_each_stack_names_the_source_the_values_and_the_due_date() {
    let text = per_stack(&[
        view("French", true, 80, 4.25, 20_000),
        view("Exam", false, 40, 6.0, 20_021),
    ]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(
        lines[0].starts_with("French (own): familiarity 80 %, difficulty 4.2"),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].starts_with("Exam (referenced): familiarity 40 %, difficulty 6.0, due 2024-"),
        "{}",
        lines[1]
    );
}

#[test]
fn a_stack_without_a_history_shows_dashes() {
    let mut r = view("French", true, 0, 0.0, 0);
    (r.percent, r.value, r.difficulty, r.due_ms) = (None, None, None, None);
    assert_eq!(
        per_stack(&[r]),
        "French (own): familiarity –, difficulty –, due –"
    );
}
