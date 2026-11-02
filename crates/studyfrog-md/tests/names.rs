#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Lists of names in one-line keys: plain, and quoted when a name needs it.

use studyfrog_md::names::{decode_names, encode_names, extra_names, with_extra_names};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn plain_lists_read_and_write_as_they_always_did() {
    assert_eq!(decode_names("Maths, Physics"), ["Maths", "Physics"]);
    assert_eq!(decode_names("  Art ,, Music  "), ["Art", "Music"]);
    assert_eq!(
        decode_names("\"Maths, Physics\""),
        ["Maths", "Physics"],
        "an old quoted line"
    );
    assert_eq!(
        encode_names(&names(&["Maths", "Physics"])),
        "Maths, Physics"
    );
    assert!(decode_names("").is_empty());
}

#[test]
fn a_name_with_a_comma_is_kept_whole() {
    let list = names(&["Prof. Dr. Künkelmann, Harry", "Maths"]);
    let text = encode_names(&list);
    assert_eq!(text, "[\"Prof. Dr. Künkelmann, Harry\", \"Maths\"]");
    assert_eq!(decode_names(&text), list);
}

#[test]
fn awkward_names_survive_a_round_trip() {
    for list in [
        names(&["He said \"hi\", twice"]),
        names(&["back\\slash", "a, b", "c"]),
        names(&["[bracketed]", "x"]),
        names(&["  padded  ", "y"]),
        names(&["Müller, Anna", "Chen, Wei", "Rivera"]),
    ] {
        assert_eq!(decode_names(&encode_names(&list)), list, "{list:?}");
    }
}

#[test]
fn quoted_lists_are_also_read_when_written_by_hand() {
    assert_eq!(decode_names("[\"A, B\",C , \"D\"]"), ["A, B", "C", "D"]);
    assert_eq!(decode_names("[]"), Vec::<String>::new());
}

#[test]
fn a_document_keeps_the_list_through_render_and_parse() {
    let doc = parse_doc("---\nid: 0000000000000000000000000A\nkind: stack\ntitle: S\nsubjects: Maths, Physics\n---\n").unwrap();
    assert_eq!(extra_names(&doc, "subjects"), ["Maths", "Physics"]);
    let list = names(&["Statistics, advanced", "Maths"]);
    let changed = with_extra_names(&doc, "subjects", &list);
    let text = render_doc(&changed).unwrap();
    assert!(
        text.contains("subjects: [\"Statistics, advanced\", \"Maths\"]"),
        "{text}"
    );
    let back = parse_doc(&text).unwrap();
    assert_eq!(extra_names(&back, "subjects"), list);
    let removed = with_extra_names(&back, "subjects", &[]);
    assert!(!render_doc(&removed).unwrap().contains("subjects"));
}
