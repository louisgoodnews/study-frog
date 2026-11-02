#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The query language: what parses, what it means as a tree, and the message for each mistake.

use studyfrog_core::query::{Cond, DateUnit, DateVal, Field, Op, Order, Query, QueryError, Val};
use studyfrog_core::query_parse::parse_query;
use studyfrog_core::query_print::print_query;
use studyfrog_core::status::Status;

fn ok(q: &str) -> Query {
    parse_query(q).unwrap_or_else(|e| panic!("{q:?}: {} (column {})", e.message, e.column))
}

fn err(q: &str) -> QueryError {
    parse_query(q).expect_err(q)
}

#[test]
fn the_examples_of_the_specification_all_parse_and_print_back_to_the_same_query() {
    for q in [
        "type = card AND status IN (new, learning) AND familiarity < 0.4",
        "stack = \"French/Verbs\" AND due <= 7d ORDER BY due",
        "tag = exam AND NOT status = done",
        "subject = Maths OR subject = Physics ORDER BY familiarity ASC, title",
        "text ~ \"Prüfung\" AND updated >= -14d",
        "priority >= high AND difficulty IS EMPTY",
        "status IN (done, paused) AND stack = French",
        "",
        "ORDER BY due DESC",
        "(type = card OR type = note) AND reviews >= 3",
        "NOT (tag = a OR tag = b)",
        "title != \"x\" AND path ~ notes/ AND created < 2026-11-18",
        "due IS NOT EMPTY AND status NOT IN (done, paused)",
        "familiarity >= 40% AND due < 2w AND updated > -1m AND created > -1y",
    ] {
        let first = ok(q);
        let printed = print_query(&first);
        assert_eq!(ok(&printed), first, "{q:?} printed as {printed:?}");
    }
}

#[test]
fn keywords_and_fields_are_not_case_sensitive_and_values_are_read_for_their_field() {
    let q = ok("TYPE = Card and Status in (NEW, learning) order by DUE desc");
    assert_eq!(
        q.cond,
        Some(Cond::And(vec![
            Cond::Cmp(
                Field::Type,
                Op::Eq,
                Val::Kind(studyfrog_core::plan::ItemKind::Card)
            ),
            Cond::In(
                Field::Status,
                vec![Val::Status(Status::New), Val::Status(Status::Learning)],
                false
            ),
        ]))
    );
    assert_eq!(
        q.order,
        [Order {
            field: Field::Due,
            descending: true
        }]
    );
    assert_eq!(
        ok("familiarity >= 40%").cond,
        Some(Cond::Cmp(Field::Familiarity, Op::Ge, Val::Num(0.4)))
    );
    assert_eq!(
        ok("due <= 7d").cond,
        Some(Cond::Cmp(
            Field::Due,
            Op::Le,
            Val::Date(DateVal::Relative(7, DateUnit::Day))
        ))
    );
    assert_eq!(
        ok("due = today").cond,
        Some(Cond::Cmp(
            Field::Due,
            Op::Eq,
            Val::Date(DateVal::Relative(0, DateUnit::Day))
        ))
    );
    assert_eq!(
        ok("updated > -2w").cond,
        Some(Cond::Cmp(
            Field::Updated,
            Op::Gt,
            Val::Date(DateVal::Relative(-2, DateUnit::Week))
        ))
    );
    assert_eq!(
        ok("title = \"a \\\"b\\\" \\\\ c\"").cond,
        Some(Cond::Cmp(
            Field::Title,
            Op::Eq,
            Val::Text("a \"b\" \\ c".into())
        ))
    );
}

#[test]
fn and_binds_tighter_than_or_and_not_binds_tightest() {
    let q = ok("tag = a OR tag = b AND NOT tag = c");
    let Some(Cond::Or(parts)) = q.cond else {
        panic!("{q:?}")
    };
    assert_eq!(parts.len(), 2);
    assert!(matches!(&parts[1], Cond::And(p) if matches!(p[1], Cond::Not(_))));
}

#[test]
fn every_mistake_of_the_specification_gives_its_message_at_its_column() {
    let cases = [
        ("type =", 7, "Expected a value after `=` at column 7"),
        (
            "status = nope",
            10,
            "Unknown status `nope` at column 10. The statuses are new, learning, reviewing, done, paused",
        ),
        (
            "due < 7d AND",
            13,
            "Expected a condition after AND at column 13",
        ),
        ("(type = card", 1, "Missing `)` for the `(` at column 1"),
        (
            "text ~ \"open",
            8,
            "The string that starts at column 8 is not closed",
        ),
        (
            "familiarity > 1.5",
            15,
            "familiarity is between 0 and 1 (column 15)",
        ),
        (
            "type > card",
            6,
            "`>` does not apply to `type`; use `=`, `!=` or `IN` (column 6)",
        ),
        (
            "priority IS EMPTY",
            10,
            "`priority` is never empty (column 10)",
        ),
        (
            "type = card status",
            13,
            "Unexpected `status` at column 13. Expected AND, OR or ORDER BY",
        ),
        (
            "type = card)",
            12,
            "Unexpected `)` at column 12: there is no `(` to close",
        ),
        ("$ = 1", 1, "Unexpected character `$` at column 1"),
        ("type IN card", 9, "Expected `(` after IN at column 9"),
        ("type IN ()", 10, "Expected a value at column 10"),
        (
            "ORDER BY text",
            10,
            "A result cannot be sorted by `text` (column 10)",
        ),
        (
            "ORDER BY title, due, type, status",
            28,
            "ORDER BY takes at most 3 keys (column 28)",
        ),
        (
            "due = 2026-13-45",
            7,
            "`2026-13-45` is not a date at column 7. Use 2026-11-18, today, 7d or -14d",
        ),
        (
            "reviews = -1",
            11,
            "reviews needs a whole number at column 11",
        ),
    ];
    for (q, column, message) in cases {
        assert_eq!(
            err(q),
            QueryError {
                column,
                message: message.into()
            },
            "{q:?}"
        );
    }
}

#[test]
fn an_unknown_field_is_offered_a_close_name_or_the_list() {
    assert_eq!(
        err("tga = x").message,
        "Unknown field `tga` at column 1. Did you mean `tag`?"
    );
    assert_eq!(
        err("statsu = done").message,
        "Unknown field `statsu` at column 1. Did you mean `status`?"
    );
    assert!(
        err("colour = red")
            .message
            .contains("The fields are text, title, type")
    );
}
