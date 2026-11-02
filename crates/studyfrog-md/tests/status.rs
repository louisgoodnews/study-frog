#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The `status` key on every kind of document.

use studyfrog_core::status::{STATUSES, Status};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;
use studyfrog_md::status::{status_of, with_status};

const DOCS: [&str; 4] = [
    "---\nid: 0000000000000000000000000A\nkind: stack\ntitle: S\n---\n",
    "---\nid: 0000000000000000000000000B\nkind: note\ntitle: N\n---\nbody\n",
    "---\nid: 0000000000000000000000000C\nkind: card\ntitle: C\n---\nq\n???\na\n",
    "---\nid: 0000000000000000000000000D\nkind: question\ntitle: Q\n---\nq\n???\na\n",
];

#[test]
fn a_status_round_trips_on_every_kind_and_nothing_else_changes() {
    for text in DOCS {
        let doc = parse_doc(text).unwrap();
        assert_eq!(status_of(&doc), None, "absent by default");
        for status in STATUSES {
            let changed = with_status(&doc, Some(status));
            let written = render_doc(&changed).unwrap();
            assert!(written.contains(&format!(
                "status: {}",
                studyfrog_core::status::status_name(status)
            )));
            assert_eq!(
                status_of(&parse_doc(&written).unwrap()),
                Some(status),
                "{text}"
            );
        }
        let removed = with_status(&with_status(&doc, Some(Status::Done)), None);
        assert_eq!(render_doc(&removed).unwrap(), render_doc(&doc).unwrap());
    }
}

#[test]
fn an_unknown_word_reads_as_none_and_stays_in_the_file_until_changed() {
    let text = "---\nid: 0000000000000000000000000C\nkind: card\ntitle: C\nstatus: finished\n---\nq\n???\na\n";
    let doc = parse_doc(text).unwrap();
    assert_eq!(status_of(&doc), None);
    assert!(
        render_doc(&doc).unwrap().contains("status: finished"),
        "kept verbatim"
    );
    let set = with_status(&doc, Some(Status::Paused));
    assert!(render_doc(&set).unwrap().contains("status: paused"));
    assert!(!render_doc(&set).unwrap().contains("finished"));
}
