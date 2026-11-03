#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Ten thousand items answer a query quickly (the budget of `docs/specs/query.md`: 50 ms; this test allows
//! a debug build ten times as much, and `cargo bench -p studyfrog-core` measures the real number).

use std::time::Instant;

use studyfrog_core::event::Difficulty;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::priority::Priority;
use studyfrog_core::query_eval::{Row, run};
use studyfrog_core::query_parse::parse_query;
use studyfrog_core::status::Status;

fn rows(n: usize) -> Vec<Row> {
    (0..n)
        .map(|i| Row {
            path: format!("stack{}/item{i}.md", i % 40),
            title: format!("Item number {i} about topic {}", i % 97),
            body: format!(
                "The back of item {i} says something about topic {} at some length.",
                i % 97
            ),
            kind: [ItemKind::Card, ItemKind::Note, ItemKind::Question][i % 3],
            status: [
                Status::New,
                Status::Learning,
                Status::Reviewing,
                Status::Done,
            ][i % 4],
            stacks: vec![format!("stack{}", i % 40), format!("Stack {}", i % 40)],
            subjects: vec![format!("subject{}", i % 7)],
            teachers: vec![format!("Teacher {}", i % 5)],
            also_in: Vec::new(),
            tags: vec![format!("tag{}", i % 11), "exam".to_owned()],
            priority: [Priority::Low, Priority::Medium, Priority::High][i % 3],
            difficulty: (i % 5 != 0).then_some(Difficulty::Medium),
            familiarity: (i % 6 != 0).then_some((i % 100) as f32 / 100.0),
            due_day: Some(20_000 + (i % 90) as i64),
            reviews: (i % 30) as u32,
            created_day: Some(19_000),
            updated_day: Some(19_900),
        })
        .collect()
}

#[test]
fn ten_thousand_items_are_filtered_and_sorted_within_the_budget() {
    let all = rows(10_000);
    for q in [
        "type = card AND status IN (new, learning) AND familiarity < 0.4 ORDER BY familiarity",
        "text ~ \"topic 42\" OR tag = tag3",
        "stack = stack7 AND due <= 30d ORDER BY due, title",
        "NOT (subject = subject1 OR teacher = \"Teacher 2\") AND reviews >= 5",
    ] {
        let query = parse_query(q).unwrap();
        let start = Instant::now();
        let found = run(&query, &all, 20_010);
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        println!("{ms:7.2} ms, {:5} found: {q}", found.len());
        assert!(ms < 500.0, "{q}: {ms} ms");
    }
}
