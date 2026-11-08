#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Item keys and colours: reading, writing, ordering, allocating, and querying by key.

use std::cmp::Ordering;

use studyfrog_core::item_color::{color_front_matter, color_rgb, normalize_color};
use studyfrog_core::item_key::{compare_keys, key_prefix, next_key, normalize_key, parse_key};
use studyfrog_core::plan::ItemKind;
use studyfrog_core::priority::Priority;
use studyfrog_core::query_eval::{Row, run};
use studyfrog_core::query_parse::parse_query;
use studyfrog_core::status::Status;

#[test]
fn keys_are_read_whatever_their_case_and_written_one_way() {
    assert_eq!(parse_key("FLASHCARD_500"), Some(("FLASHCARD", 500)));
    assert_eq!(parse_key("note_12"), Some(("NOTE", 12)));
    assert_eq!(normalize_key(" stack_007 "), Some("STACK_7".to_owned()));
    for bad in [
        "CARD_1", "NOTE_", "NOTE_0", "NOTE_-1", "NOTE_1a", "NOTE1", "", "_1",
    ] {
        assert_eq!(parse_key(bad), None, "{bad}");
    }
    assert_eq!(key_prefix("card"), Some("FLASHCARD"));
    assert_eq!(key_prefix("question"), Some("QUESTION"));
    assert_eq!(key_prefix("diary"), None);
}

#[test]
fn the_next_key_follows_the_highest_number_taken_or_handed_out() {
    let taken = ["NOTE_3", "NOTE_10", "FLASHCARD_99", "junk"];
    assert_eq!(next_key("NOTE", taken, 0), "NOTE_11");
    assert_eq!(next_key("NOTE", taken, 40), "NOTE_41");
    assert_eq!(next_key("QUESTION", taken, 0), "QUESTION_1");
    assert_eq!(next_key("FLASHCARD", taken, 0), "FLASHCARD_100");
}

#[test]
fn keys_sort_by_kind_then_number() {
    assert_eq!(compare_keys("NOTE_2", "NOTE_10"), Ordering::Less);
    assert_eq!(compare_keys("FLASHCARD_9", "NOTE_1"), Ordering::Less);
    assert_eq!(compare_keys("NOTE_1", "zzz"), Ordering::Less);
}

#[test]
fn colours_are_palette_words_or_codes() {
    assert_eq!(normalize_color("Sage"), Some("sage".to_owned()));
    assert_eq!(normalize_color("blue"), Some("sky".to_owned()));
    assert_eq!(normalize_color("#4a90e2"), Some("#4A90E2".to_owned()));
    assert_eq!(normalize_color("\"#4A90E2\""), Some("#4A90E2".to_owned()));
    assert_eq!(color_rgb("#4A90E2"), Some([0x4A, 0x90, 0xE2]));
    assert!(color_rgb("amber").is_some());
    for bad in ["#123", "#GGGGGG", "chartreuse", ""] {
        assert_eq!(normalize_color(bad), None, "{bad}");
    }
    assert_eq!(color_front_matter("#4a90e2").unwrap(), "\"#4A90E2\"");
    assert_eq!(color_front_matter("ROSE").unwrap(), "rose");
}

fn row(path: &str, key: Option<&str>) -> Row {
    Row {
        path: path.into(),
        title: path.into(),
        body: String::new(),
        kind: ItemKind::Card,
        status: Status::New,
        stacks: Vec::new(),
        also_in: Vec::new(),
        subjects: Vec::new(),
        teachers: Vec::new(),
        tags: Vec::new(),
        priority: Priority::Medium,
        difficulty: None,
        familiarity: None,
        due_day: None,
        reviews: 0,
        created_day: None,
        updated_day: None,
        key: key.map(str::to_owned),
    }
}

fn found(query: &str) -> Vec<String> {
    let rows = [
        row("a.md", Some("FLASHCARD_10")),
        row("b.md", Some("FLASHCARD_2")),
        row("c.md", Some("NOTE_1")),
        row("d.md", None),
    ];
    let q = parse_query(query).expect(query);
    run(&q, &rows, 0)
        .into_iter()
        .map(|i| rows[i].path.clone())
        .collect()
}

#[test]
fn a_query_filters_and_sorts_by_key() {
    assert_eq!(found("key = FLASHCARD_2"), ["b.md"]);
    assert_eq!(found("key = flashcard_002"), ["b.md"]);
    assert_eq!(found("key ~ FLASHCARD"), ["a.md", "b.md"]);
    assert_eq!(found("key != NOTE_1"), ["a.md", "b.md", "d.md"]);
    assert_eq!(found("key IN (NOTE_1, FLASHCARD_10)"), ["a.md", "c.md"]);
    assert_eq!(found("key IS EMPTY"), ["d.md"]);
    assert_eq!(
        found("key IS NOT EMPTY ORDER BY key"),
        ["b.md", "a.md", "c.md"]
    );
    assert_eq!(found("ORDER BY key DESC"), ["c.md", "a.md", "b.md", "d.md"]);
}
