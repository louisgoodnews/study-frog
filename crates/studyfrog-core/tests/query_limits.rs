#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The query language: its limits, and that hostile text is only ever a value.

use studyfrog_core::query::{Cond, Field, Op, QueryError, Val};
use studyfrog_core::query_parse::parse_query;
use studyfrog_core::query_print::print_query;

fn ok(q: &str) -> studyfrog_core::query::Query {
    parse_query(q).unwrap_or_else(|e| panic!("{q:?}: {} (column {})", e.message, e.column))
}

fn err(q: &str) -> QueryError {
    parse_query(q).expect_err(q)
}

#[test]
fn the_limits_hold() {
    let long = format!("title = \"{}\"", "a".repeat(1_989));
    assert_eq!(long.chars().count(), 1_999);
    ok(&long);
    let too_long = format!("title = \"{}\"", "a".repeat(2_000));
    assert_eq!(err(&too_long).message, "A query is at most 2000 characters");
    ok(&format!("{}type = card{}", "(".repeat(20), ")".repeat(20)));
    assert!(
        err(&format!("{}type = card{}", "(".repeat(21), ")".repeat(21)))
            .message
            .starts_with("Too many nested")
    );
    assert!(
        err(&format!("{}type = card", "NOT ".repeat(25)))
            .message
            .starts_with("Too many nested")
    );
    ok("ORDER BY title, due, type");
}

#[test]
fn text_that_looks_like_sql_or_a_query_is_only_ever_a_value() {
    for hostile in [
        "x'; DROP TABLE item; --",
        "%_\\",
        "a\" OR 1=1 --",
        ") OR (",
        "ORDER BY",
        "\u{0}\u{1f}",
    ] {
        let quoted = hostile.replace('\\', "\\\\").replace('"', "\\\"");
        let q = ok(&format!("title = \"{quoted}\""));
        assert_eq!(
            q.cond,
            Some(Cond::Cmp(Field::Title, Op::Eq, Val::Text(hostile.into()))),
            "{hostile:?}"
        );
        assert_eq!(ok(&print_query(&q)), q);
    }
}
