#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The saved filters file.

use studyfrog_config::filters::{
    SavedFilter, name_problem, parse_filters, render_filters, with_filter, without_filter,
};

fn f(name: &str, query: &str) -> SavedFilter {
    SavedFilter {
        name: name.into(),
        query: query.into(),
    }
}

#[test]
fn a_list_survives_a_round_trip_with_quotes_and_backslashes() {
    let list = vec![
        f("Due soon", "due <= 7d ORDER BY due"),
        f("Quotes \"here\"", "title ~ \"a \\\\ b\""),
        f("Umlaute äöü", ""),
    ];
    assert_eq!(parse_filters(&render_filters(&list)).unwrap(), list);
    assert!(parse_filters("").unwrap().is_empty());
}

#[test]
fn bad_files_are_refused_with_a_reason() {
    assert!(
        parse_filters("[[filter]]\nname = \"a\"\n").is_err(),
        "no query"
    );
    assert!(parse_filters("[[filter]]\nname = \"a\"\nquery = \"\"\nx = 1\n").is_err());
    let twice = "[[filter]]\nname = \"A\"\nquery = \"\"\n[[filter]]\nname = \"a\"\nquery = \"\"\n";
    assert!(parse_filters(twice).is_err(), "names differ only in case");
    assert!(parse_filters("[[filter]]\nname = \" \"\nquery = \"\"\n").is_err());
}

#[test]
fn saving_under_a_known_name_replaces_and_removing_ignores_case() {
    let list = vec![f("One", "a"), f("Two", "b")];
    let next = with_filter(&list, f(" one ", "c")).unwrap();
    assert_eq!(next, [f("one", "c"), f("Two", "b")]);
    let next = with_filter(&list, f("Three", "d")).unwrap();
    assert_eq!(next.len(), 3);
    assert!(with_filter(&list, f("  ", "d")).is_err());
    assert_eq!(without_filter(&list, "TWO"), [f("One", "a")]);
    assert!(name_problem(&"x".repeat(81)).is_some() && name_problem("a\nb").is_some());
}
