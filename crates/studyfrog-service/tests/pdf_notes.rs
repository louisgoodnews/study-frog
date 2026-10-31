#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Notes made from PDF text keep a link back to the page.

#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::path::PathBuf;

use studyfrog_service::create::IdSource;
use studyfrog_service::pdf_notes::{NoteRequest, note_from_pdf, reflow};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::pdf_notes::{items_made_from, split_source};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn session(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-pnotes-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("bio")).unwrap();
    fs::write(
        dir.join("bio/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: Bio\n---\n",
    )
    .unwrap();
    fs::write(dir.join("bio/three.pdf"), pdfmake::three()).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn ids(n: u8) -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000,
        random: [n; 10],
    }
}

fn req<'a>(first: usize, last: usize, selection: Option<&'a str>) -> NoteRequest<'a> {
    NoteRequest {
        pdf: "bio/three.pdf",
        first,
        last,
        selection,
        folder: "bio",
        title: "",
        pictures: &[],
    }
}

#[test]
fn text_broken_into_lines_becomes_running_text_with_headings_and_lists_kept() {
    assert_eq!(
        reflow("The cell divi-\nsion starts\nhere.\n\nNext para"),
        "The cell division starts here.\n\nNext para"
    );
    assert_eq!(reflow("# Title\n\n- one\n- two"), "# Title\n\n- one\n- two");
    assert_eq!(
        reflow("well-\nKnown thing"),
        "well- Known thing",
        "a capital after a hyphen keeps the hyphen"
    );
    assert_eq!(reflow("  \n\n"), "");
    assert_eq!(reflow("a\r\nb"), "a b");
}

#[test]
fn pages_become_a_note_with_a_source_key_and_the_page_lists_it() {
    let (dir, mut s) = session("pages");
    let (_, made) = note_from_pdf(&mut s, &req(1, 2, None), ids(1)).unwrap();
    assert!(made.title.contains("pages 2 to 3"), "{}", made.title);
    let text = fs::read_to_string(dir.0.join(&made.path)).unwrap();
    assert!(text.contains("source: \"bio/three.pdf#page=2\""), "{text}");
    assert!(
        text.contains("Beta") && text.contains("Gamma") && !text.contains("Alpha"),
        "{text}"
    );
    let found = items_made_from(&s.conn, "bio/three.pdf", Some(1)).unwrap();
    assert_eq!(
        (found.len(), found[0].kind.as_str(), found[0].page),
        (1, "note", Some(1))
    );
    assert!(
        items_made_from(&s.conn, "bio/three.pdf", Some(0))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        items_made_from(&s.conn, "bio/three.pdf", None)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_selection_is_used_instead_of_the_pages_and_a_title_can_be_given() {
    let (dir, mut s) = session("selection");
    let r = NoteRequest {
        title: "My excerpt",
        ..req(0, 0, Some("only this\npart"))
    };
    let (_, made) = note_from_pdf(&mut s, &r, ids(2)).unwrap();
    assert_eq!(made.title, "My excerpt");
    let text = fs::read_to_string(dir.0.join(&made.path)).unwrap();
    assert!(
        text.contains("only this part") && !text.contains("Alpha"),
        "{text}"
    );
}

#[test]
fn bad_requests_are_refused() {
    let (_d, mut s) = session("bad");
    assert!(
        note_from_pdf(&mut s, &req(0, 9, None), ids(3)).is_err(),
        "no such page"
    );
    let outside = NoteRequest {
        pdf: "../x.pdf",
        ..req(0, 0, None)
    };
    assert!(note_from_pdf(&mut s, &outside, ids(3)).is_err());
    let not_stack = NoteRequest {
        folder: "nowhere",
        ..req(0, 0, None)
    };
    assert!(note_from_pdf(&mut s, &not_stack, ids(3)).is_err());
}

#[test]
fn a_source_splits_into_path_and_page() {
    assert_eq!(split_source("a/b.pdf#page=12"), ("a/b.pdf", Some(11)));
    assert_eq!(split_source("a/b.pdf"), ("a/b.pdf", None));
    assert_eq!(split_source("a/b.pdf#page=0"), ("a/b.pdf", None));
}

#[test]
fn pictures_are_listed_at_the_end_of_the_note() {
    let (dir, mut s) = session("pics");
    let pictures = vec!["assets/fig-1.png".to_owned()];
    let r = NoteRequest {
        pictures: &pictures,
        ..req(0, 0, None)
    };
    let (_, made) = note_from_pdf(&mut s, &r, ids(9)).unwrap();
    let text = fs::read_to_string(dir.0.join(&made.path)).unwrap();
    assert!(
        text.trim_end().ends_with("![fig-1](assets/fig-1.png)"),
        "{text}"
    );
}

#[test]
fn a_card_and_a_question_are_made_with_the_page_as_their_source() {
    use studyfrog_service::create_more::NewKind;
    use studyfrog_service::pdf_notes::{ItemRequest, item_from_pdf};
    let (dir, mut s) = session("items");
    let base = ItemRequest {
        kind: NewKind::Card,
        pdf: "bio/three.pdf",
        page: 2,
        front: "What is third?",
        back: "Gamma",
        folder: "bio",
    };
    let (_, card) = item_from_pdf(&mut s, &base, ids(5)).unwrap();
    let text = fs::read_to_string(dir.0.join(&card.path)).unwrap();
    assert!(
        text.contains("source: \"bio/three.pdf#page=3\"") && text.contains("Gamma"),
        "{text}"
    );
    let q = ItemRequest {
        kind: NewKind::Question,
        front: "Name the second letter",
        back: "Beta",
        page: 1,
        ..base
    };
    item_from_pdf(&mut s, &q, ids(6)).unwrap();
    let from = items_made_from(&s.conn, "bio/three.pdf", None).unwrap();
    let kinds: Vec<&str> = from.iter().map(|i| i.kind.as_str()).collect();
    assert_eq!(kinds.len(), 2);
    assert!(kinds.contains(&"card") && kinds.contains(&"question"));
    assert!(
        item_from_pdf(&mut s, &ItemRequest { back: "", ..base }, ids(7)).is_err(),
        "both sides are needed"
    );
    assert!(item_from_pdf(&mut s, &ItemRequest { page: 9, ..base }, ids(8)).is_err());
    assert!(
        item_from_pdf(
            &mut s,
            &ItemRequest {
                kind: NewKind::Note,
                ..base
            },
            ids(8)
        )
        .is_err()
    );
}
