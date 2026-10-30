#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! One-line front-matter keys the format does not type, such as `priority`.

use studyfrog_md::extra::{extra_value, with_extra};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;

const CARD: &str = "---\nid: 0000000000000000000000000A\nkind: card\ntitle: C\npriority: high\nmood: \"calm\"\n---\nq\n???\na\n";

#[test]
fn a_key_is_read_without_quotes_and_missing_keys_are_none() {
    let doc = parse_doc(CARD).unwrap();
    assert_eq!(extra_value(&doc, "priority").as_deref(), Some("high"));
    assert_eq!(extra_value(&doc, "mood").as_deref(), Some("calm"));
    assert_eq!(extra_value(&doc, "difficulty"), None);
}

#[test]
fn setting_replaces_adds_and_removes_and_the_other_keys_survive_a_round_trip() {
    let doc = parse_doc(CARD).unwrap();
    let changed = with_extra(&doc, "priority", Some("lowest"));
    let added = with_extra(&changed, "difficulty", Some("easy"));
    let text = render_doc(&added).unwrap();
    let back = parse_doc(&text).unwrap();
    assert_eq!(extra_value(&back, "priority").as_deref(), Some("lowest"));
    assert_eq!(extra_value(&back, "difficulty").as_deref(), Some("easy"));
    assert_eq!(
        extra_value(&back, "mood").as_deref(),
        Some("calm"),
        "unrelated keys are kept"
    );
    let removed = with_extra(&back, "priority", None);
    assert_eq!(extra_value(&removed, "priority"), None);
    assert!(!render_doc(&removed).unwrap().contains("priority"));
}
