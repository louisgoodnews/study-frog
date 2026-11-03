#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `referenced_by` (TASK-0211): read, written, kept through a round trip, absent in old files.

use proptest::prelude::*;
use studyfrog_md::parse::parse_doc;
use studyfrog_md::refs::{references_of, with_references};
use studyfrog_md::render::render_doc;

const NOTE: &str = "---\nid: 0000000000000000000000000A\nkind: note\ntitle: A\nreferenced_by: [00000000000000000000000001, 00000000000000000000000002]\ncolour: red\n---\nbody\n";

#[test]
fn the_list_is_read_in_order_and_unknown_keys_stay() {
    let doc = parse_doc(NOTE).unwrap();
    assert_eq!(
        references_of(&doc),
        ["00000000000000000000000001", "00000000000000000000000002"]
    );
    let again = render_doc(&doc).unwrap();
    assert!(
        again.contains("colour: red") && again.contains("referenced_by: ["),
        "{again}"
    );
}

#[test]
fn old_files_have_none_and_setting_and_clearing_work() {
    let old = "---\nid: 0000000000000000000000000A\nkind: card\ntitle: A\n---\nq\n???\na\n";
    let doc = parse_doc(old).unwrap();
    assert!(references_of(&doc).is_empty());
    assert_eq!(
        render_doc(&doc).unwrap(),
        old,
        "a file without the key is written back unchanged"
    );
    let with = with_references(&doc, &["00000000000000000000000003".to_owned()]);
    assert_eq!(references_of(&with), ["00000000000000000000000003"]);
    assert!(
        render_doc(&with)
            .unwrap()
            .contains("referenced_by: [00000000000000000000000003]")
    );
    assert!(references_of(&with_references(&with, &[])).is_empty());
}

#[test]
fn quotes_blanks_and_a_single_bare_id_are_read() {
    let doc = parse_doc(&NOTE.replace(
        "[00000000000000000000000001, 00000000000000000000000002]",
        "[\"AAA\", 'BBB' ,  ]",
    ))
    .unwrap();
    assert_eq!(references_of(&doc), ["AAA", "BBB"]);
    let doc = parse_doc(&NOTE.replace(
        "[00000000000000000000000001, 00000000000000000000000002]",
        "CCC",
    ))
    .unwrap();
    assert_eq!(references_of(&doc), ["CCC"]);
}

proptest! {
    #[test]
    fn any_list_of_ids_survives_a_write_and_a_read(ids in proptest::collection::vec("[0-9A-HJKMNP-TV-Z]{26}", 0..6)) {
        let doc = parse_doc(NOTE).unwrap();
        let written = render_doc(&with_references(&doc, &ids)).unwrap();
        let back = parse_doc(&written).unwrap();
        prop_assert_eq!(references_of(&back), ids);
        prop_assert!(written.contains("colour: red"));
    }
}
