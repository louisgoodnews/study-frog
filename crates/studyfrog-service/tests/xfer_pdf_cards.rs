#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! PDF import as cards (STORY-0191): one card per page with text, and the pages without text listed as
//! left out in the preview.

use std::fs;
use std::path::PathBuf;

use studyfrog_md::ulid::parse_ulid;
use studyfrog_service::pdf_export::{ExportTarget, default_pdf_export_options, export_pdf};
use studyfrog_service::session::open_vault;
use studyfrog_service::xfer_pdf::{from_pdf, page_cards};
use studyfrog_store::reindex::reindex;

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn each_page_with_text_is_a_card_and_the_others_are_left_out() {
    let pages = [
        (1, "Reliability\nConsistency over repeated tests."),
        (2, "   \n"),
        (
            3,
            "Validity\nDoes the test measure the construct?\nSecond line.",
        ),
    ];
    let (cards, warnings) = page_cards(&pages);
    assert_eq!(cards.len(), 2, "pages 1 and 3 have text");
    assert_eq!(cards[0].title, "Page 1");
    assert_eq!(cards[1].title, "Page 3");
    assert_eq!(warnings, ["page 2 has too little text for a card"]);
}

#[test]
fn a_real_pdf_gives_one_card_for_its_text_page() {
    let dir = std::env::temp_dir().join(format!("studyfrog-xfer-pdf-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let _guard = Dir(dir.clone());
    let (mut s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let uid = parse_ulid("00000000000000000000000001").unwrap();
    fs::write(
        dir.join("note.md"),
        "---\nid: 00000000000000000000000001\nkind: note\ntitle: Biology\n---\nMitochondria produce ATP.\nThey have their own DNA.\n",
    )
    .unwrap();
    reindex(&mut s.conn, &s.root).unwrap();
    let pdf = dir.join("note.pdf");
    export_pdf(
        &s,
        &ExportTarget::SingleItem(uid),
        &default_pdf_export_options(),
        &pdf,
    )
    .unwrap();

    let bundle = from_pdf(&fs::read(&pdf).unwrap()).unwrap();
    assert!(!bundle.items.is_empty(), "the text page becomes a card");
    assert!(bundle.items.iter().all(|i| i.title.starts_with("Page ")));
}
