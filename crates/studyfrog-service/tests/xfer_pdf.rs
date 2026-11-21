#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Importing a PDF as cards: one card per page with text, read through the import adapters.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::plan::ItemKind;
use studyfrog_md::ulid::parse_ulid;
use studyfrog_service::pdf_export::{ExportTarget, default_pdf_export_options, export_pdf};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_service::xfer_adapters::{adapter_for, read_source};
use studyfrog_service::xfer_pdf::{name_cards, page_cards};
use studyfrog_store::reindex::reindex;

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_page_becomes_a_card_with_its_first_line_as_the_front() {
    let pages = [
        (
            1,
            "Cell biology\nMitochondria produce ATP.\nThey have their own DNA.",
        ),
        (2, "   \n\n"),
        (3, "Only one line here"),
    ];
    let (items, warnings) = page_cards(&pages);
    assert_eq!(items.len(), 1);
    let card = &items[0];
    assert_eq!(card.kind, ItemKind::Card);
    assert_eq!(card.title, "Page 1");
    assert_eq!(card.text, "Cell biology");
    assert_eq!(
        card.answer,
        "Mitochondria produce ATP.\nThey have their own DNA."
    );
    assert_eq!(
        warnings,
        [
            "page 2 has too little text for a card",
            "page 3 has too little text for a card"
        ]
    );
}

#[test]
fn long_pages_are_clipped_to_what_a_card_holds() {
    let long = format!("{}\n{}", "f".repeat(500), "b".repeat(3000));
    let (items, _) = page_cards(&[(1, long.as_str())]);
    assert_eq!(items[0].text.chars().count(), 200);
    assert_eq!(items[0].answer.chars().count(), 2000);
}

#[test]
fn cards_take_the_file_name_so_several_pdfs_stay_apart() {
    let (mut items, _) = page_cards(&[(4, "Front\nBack")]);
    let mut bundle = studyfrog_service::xfer_model::Bundle {
        items: std::mem::take(&mut items),
        ..Default::default()
    };
    name_cards(&mut bundle, "/home/me/Lecture 3.pdf");
    assert_eq!(bundle.items[0].title, "Lecture 3: Page 4");
}

#[test]
fn a_pdf_is_recognised_and_read_into_cards() {
    assert_eq!(
        adapter_for("lecture.PDF").map(|a| a.name),
        Some(studyfrog_service::xfer_pdf::PDF_NAME)
    );
    let dir = std::env::temp_dir().join(format!("studyfrog-xfer-pdf-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let _guard = Dir(dir.clone());
    let (mut session, _): (VaultSession, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let uid = parse_ulid("00000000000000000000000007").unwrap();
    let md = "---\nid: 00000000000000000000000007\nkind: note\ntitle: Cellular Biology\n---\n# Cellular Biology\n\nMitochondria produce ATP for cellular work.\nThe cell membrane controls what enters.\n";
    fs::write(dir.join("biology.md"), md).unwrap();
    reindex(&mut session.conn, &session.root).unwrap();
    let pdf = dir.join("exported.pdf");
    export_pdf(
        &session,
        &ExportTarget::SingleItem(uid),
        &default_pdf_export_options(),
        &pdf,
    )
    .unwrap();

    let (bundle, adapter) = read_source(&pdf).unwrap();
    assert_eq!(adapter, studyfrog_service::xfer_pdf::PDF_NAME);
    assert!(!bundle.items.is_empty(), "the exported page has text");
    assert!(bundle.items.iter().all(|i| i.kind == ItemKind::Card));
    assert!(
        bundle.items[0].title.starts_with("exported: Page "),
        "{}",
        bundle.items[0].title
    );
}
