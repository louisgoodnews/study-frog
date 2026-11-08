#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Anything a user can type is either a query or a message: it never panics, and what parses can be printed back.

use proptest::prelude::*;
use studyfrog_core::query_eval::{Row, run};
use studyfrog_core::query_parse::parse_query;
use studyfrog_core::query_print::print_query;

const VOCAB: [&str; 52] = [
    "type",
    "status",
    "stack",
    "subject",
    "teacher",
    "tag",
    "priority",
    "difficulty",
    "familiarity",
    "due",
    "reviews",
    "created",
    "updated",
    "path",
    "text",
    "title",
    "=",
    "!=",
    "<",
    "<=",
    ">",
    ">=",
    "~",
    "!~",
    "AND",
    "OR",
    "NOT",
    "IN",
    "IS",
    "EMPTY",
    "ORDER",
    "BY",
    "ASC",
    "DESC",
    "(",
    ")",
    ",",
    "\"x y\"",
    "card",
    "new",
    "done",
    "high",
    "0.4",
    "40%",
    "7d",
    "-14d",
    "today",
    "2026-11-18",
    "\"",
    "\\",
    "é",
    "'",
];

proptest! {
    #[test]
    fn arbitrary_text_never_panics_and_a_parsed_query_prints_back_to_itself(text in "\\PC{0,120}") {
        if let Ok(q) = parse_query(&text) {
            prop_assert_eq!(parse_query(&print_query(&q)), Ok(q));
        }
    }

    #[test]
    fn queries_built_from_the_words_of_the_language_never_panic_and_errors_point_inside_the_text(
        words in proptest::collection::vec(0_usize..VOCAB.len(), 0..30),
    ) {
        let text = words.iter().map(|i| VOCAB[*i]).collect::<Vec<_>>().join(" ");
        match parse_query(&text) {
            Ok(q) => prop_assert_eq!(parse_query(&print_query(&q)), Ok(q)),
            Err(e) => {
                prop_assert!(e.column >= 1 && e.column <= text.chars().count() + 1, "{} in {text:?}", e.column);
                prop_assert!(!e.message.is_empty());
            }
        }
    }

    #[test]
    fn a_parsed_query_runs_over_any_rows_and_never_returns_a_row_twice(
        words in proptest::collection::vec(0_usize..VOCAB.len(), 0..20),
    ) {
        let text = words.iter().map(|i| VOCAB[*i]).collect::<Vec<_>>().join(" ");
        if let Ok(q) = parse_query(&text) {
            let rows = vec![blank("a.md"), blank("b.md"), blank("c.md")];
            let found = run(&q, &rows, 20_000);
            let mut sorted = found.clone();
            sorted.sort_unstable();
            sorted.dedup();
            prop_assert_eq!(sorted.len(), found.len());
            prop_assert!(found.iter().all(|i| *i < rows.len()));
        }
    }
}

fn blank(path: &str) -> Row {
    use studyfrog_core::plan::ItemKind;
    use studyfrog_core::priority::Priority;
    use studyfrog_core::status::Status;
    Row {
        path: path.into(),
        title: path.into(),
        body: String::new(),
        kind: ItemKind::Card,
        status: Status::New,
        stacks: Vec::new(),
        subjects: Vec::new(),
        teachers: Vec::new(),
        also_in: Vec::new(),
        tags: Vec::new(),
        priority: Priority::Medium,
        difficulty: None,
        familiarity: None,
        due_day: None,
        reviews: 0,
        created_day: None,
        updated_day: None,
        key: None,
    }
}
