#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Suggestions while a query is typed.

use studyfrog_core::query_complete::{apply, complete};

fn texts(q: &str) -> Vec<String> {
    complete(q).into_iter().map(|c| c.text).collect()
}

#[test]
fn an_empty_query_offers_the_fields_and_a_prefix_narrows_them() {
    let all = texts("");
    assert_eq!(all.len(), 8, "at most eight at a time");
    assert!(all.contains(&"text".to_owned()) && all.contains(&"title".to_owned()));
    assert_eq!(texts("fam"), ["familiarity"]);
    assert_eq!(texts("ST"), ["status", "stack"]);
    assert_eq!(texts("type = card AND st"), ["status", "stack"]);
    assert!(texts("zzz").is_empty());
}

#[test]
fn after_a_field_come_its_operators() {
    assert_eq!(texts("type "), ["=", "!=", "IN"]);
    assert_eq!(
        texts("difficulty "),
        ["=", "!=", "<", "<=", ">", ">=", "IN", "IS"]
    );
    assert_eq!(texts("text "), ["~", "!~"]);
    assert_eq!(texts("due "), ["=", "!=", "<", "<=", ">", ">=", "IS"]);
    assert_eq!(texts("type I"), ["IN"], "a word being typed is a prefix");
}

#[test]
fn after_an_operator_come_the_values_of_the_field() {
    assert_eq!(texts("type = "), ["note", "card", "question"]);
    assert_eq!(texts("status = l"), ["learning"]);
    assert_eq!(
        texts("status IN ("),
        ["new", "learning", "reviewing", "done", "paused"]
    );
    assert_eq!(texts("status IN (new, d"), ["done"]);
    assert_eq!(texts("priority >= h"), ["high", "highest"]);
    assert_eq!(texts("due <= "), ["today", "7d", "-14d", "2w", "1m"]);
    assert!(
        texts("title = ").is_empty(),
        "free text has nothing to suggest"
    );
}

#[test]
fn after_a_value_come_the_keywords_and_after_by_the_fields() {
    assert_eq!(texts("type = card "), ["AND", "OR", "ORDER BY"]);
    assert_eq!(texts("type = card O"), ["OR", "ORDER BY"]);
    assert_eq!(
        texts("type = card AND "),
        texts("").into_iter().collect::<Vec<_>>()
    );
    assert!(texts("ORDER BY d").contains(&"due".to_owned()));
    assert!(texts("ORDER BY title, d").contains(&"due".to_owned()));
    assert_eq!(texts("tag IS "), ["EMPTY", "NOT"]);
    assert_eq!(texts("NOT "), texts(""));
}

#[test]
fn a_completion_replaces_the_word_being_typed_and_leaves_a_space() {
    let c = complete("type = card AND st").remove(0);
    assert_eq!(c.from, 16);
    assert_eq!(apply("type = card AND st", &c), "type = card AND status ");
    let c = complete("type ").remove(0);
    assert_eq!(apply("type ", &c), "type = ");
    let c = complete("status IN ").remove(0);
    assert_eq!(apply("status IN ", &c), "status IN (");
    let c = complete("").remove(0);
    assert_eq!(apply("", &c), format!("{} ", c.text));
}

#[test]
fn broken_text_gives_no_suggestions_instead_of_an_error() {
    assert!(complete("title = \"open").is_empty());
    assert!(complete("$$$").is_empty());
}
