#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Universal Import Hub service tests: Markdown parsing, ImportTarget routing, VaultRootNotes, NewStack, and PDFs.

#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::path::{Path, PathBuf};

use studyfrog_core::plan::ItemKind;
use studyfrog_service::create::IdSource;
use studyfrog_service::pdfs::pdf_library;
use studyfrog_service::session::{ServiceError, VaultSession, open_vault};
use studyfrog_service::xfer_adapters::{SourceFormat, detect_format, format_label, read_source};
use studyfrog_service::xfer_import::{
    ImportDone, ImportTarget, Policy, apply_import_to, counts, plan_import_to,
};
use studyfrog_service::xfer_model::Bundle;
use studyfrog_service::xfer_pdf::import_pdf_to;
use studyfrog_store::items::list_items;

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn ids(seed: u8) -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000,
        random: [seed; 10],
    }
}

fn setup(label: &str) -> (Dir, VaultSession, PathBuf) {
    let dir = std::env::temp_dir().join(format!("studyfrog-hub-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let (vault, src) = (dir.join("vault"), dir.join("src"));
    fs::create_dir_all(vault.join("french")).unwrap();
    fs::create_dir_all(&src).unwrap();
    fs::write(
        vault.join("french/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: French\n---\n",
    )
    .unwrap();
    let (session, _) = open_vault(vault.to_str().unwrap()).unwrap();
    (Dir(dir), session, src)
}

fn write_and_read(dir: &Path, name: &str, text: &str) -> (PathBuf, Bundle) {
    let p = dir.join(name);
    fs::write(&p, text).unwrap();
    let (b, _) = read_source(&p).unwrap();
    (p, b)
}

fn apply_ok(s: &mut VaultSession, b: &Bundle, target: ImportTarget, seed: u8) -> ImportDone {
    apply_import_to(s, b, &target, (Policy::Skip, false), ids(seed)).unwrap()
}

#[test]
fn markdown_notes_frontmatter_decks_and_errors_parse_cleanly() {
    let (_d, _s, src) = setup("md");
    notes_and_decks(&src);
    questions_and_errors(&src);
}

fn notes_and_decks(src: &Path) {
    let plain = src.join("plain-note.md");
    fs::write(&plain, "# Lecture\nRemember #biology and #cell-wall.\n").unwrap();
    assert_eq!(detect_format(&plain), Some(SourceFormat::Markdown));
    assert_eq!(format_label(SourceFormat::Markdown), "Markdown");
    let (b, adapter) = read_source(&plain).unwrap();
    assert_eq!(
        (adapter, b.items.len(), b.items[0].kind),
        ("Markdown", 1, ItemKind::Note)
    );
    assert!(b.items[0].title == "plain-note" && b.items[0].tags == ["biology", "cell-wall"]);

    let (_, b) = write_and_read(
        src,
        "obs.md",
        "---\ntitle: Cell Notes\ntags: [bio, study]\ncssclass: wide\n---\n> [!note]\nBody.\n",
    );
    assert!(b.items[0].title == "Cell Notes" && b.items[0].tags == ["bio", "study"]);
    assert_eq!(
        b.items[0].extra,
        [("cssclass".to_owned(), "wide".to_owned())]
    );
    assert!(b.warnings.iter().any(|w| w.contains("callouts")));

    let (_, b) = write_and_read(
        src,
        "deck.md",
        "```md\n---\n???\n```\nFront 1\n???\nBack 1\n---\nFront 2\n???\nBack 2\n",
    );
    assert_eq!(b.items.len(), 2);
    assert!(b.items.iter().all(|i| i.kind == ItemKind::Card));
    assert!(b.items[0].text.contains("```md") && b.items[1].answer == "Back 2");
}

fn questions_and_errors(src: &Path) {
    let (_, b) = write_and_read(
        src,
        "qa.md",
        "Q: Capital of France?\nA: Paris\n\nQ: 2+2?\nA: 4\n",
    );
    assert_eq!(
        (
            b.items.len(),
            b.items[0].text.as_str(),
            b.items[0].answer.as_str()
        ),
        (2, "Capital of France?", "Paris")
    );

    let (_, b) = write_and_read(
        src,
        "hand.md",
        "---\nkind: card\ntitle: Greet\n---\nSalut\n???\nHi\n",
    );
    assert_eq!((b.items[0].kind, b.items[0].id), (ItemKind::Card, None));

    let (_, b) = write_and_read(
        src,
        "sf.md",
        "---\nid: 00000000000000000000000099\nkind: question\ntitle: Q1\npriority: high\n---\nWhy?\n???\nBecause.\n",
    );
    assert_eq!(
        (b.items[0].kind, b.items[0].id.is_some()),
        (ItemKind::Question, true)
    );
    assert_eq!(b.items[0].priority.as_deref(), Some("high"));

    let stack_md = src.join("_stack.md");
    fs::write(
        &stack_md,
        "---\nid: 00000000000000000000000002\nkind: stack\ntitle: Bad\n---\n",
    )
    .unwrap();
    assert!(read_source(&stack_md).unwrap_err().contains("_stack.md"));
    let empty = src.join("empty.md");
    fs::write(&empty, "   \n").unwrap();
    assert!(read_source(&empty).unwrap_err().contains("empty"));
}

#[test]
fn targets_support_existing_stack_new_stack_and_vault_root_inbox() {
    let (d, mut s, src) = setup("targets");
    let vault = d.0.join("vault");

    let (_, nb) = write_and_read(&src, "root-note.md", "Unstacked thoughts.\n");
    let plan = plan_import_to(&s, &nb, &ImportTarget::VaultRootNotes).unwrap();
    assert_eq!((counts(&plan), plan.new_stacks), ((1, 0, 0), 0));
    let done = apply_ok(&mut s, &nb, ImportTarget::VaultRootNotes, 1);
    assert_eq!((done.created, done.info.overview.inbox_items), (1, 1));
    assert!(vault.join("root-note.md").is_file() && !vault.join("_stack.md").exists());
    let inbox = list_items(&s.conn, None).unwrap();
    assert_eq!((inbox.len(), inbox[0].title.as_str()), (1, "root-note"));

    let (_, cb) = write_and_read(&src, "cards.md", "One\n???\nUno\n");
    apply_ok(&mut s, &cb, ImportTarget::VaultRootNotes, 2);
    assert_eq!(
        counts(&plan_import_to(&s, &cb, &ImportTarget::VaultRootNotes).unwrap()),
        (0, 1, 0)
    );

    let err = plan_import_to(&s, &cb, &ImportTarget::NewStack("   ".into())).unwrap_err();
    assert!(matches!(err, ServiceError::Invalid(_)));

    let done1 = apply_ok(
        &mut s,
        &cb,
        ImportTarget::NewStack("Organic Chemistry".into()),
        3,
    );
    assert_eq!(done1.created, 1);
    assert!(
        fs::read_to_string(vault.join("organic-chemistry/_stack.md"))
            .unwrap()
            .contains("title: Organic Chemistry")
    );

    let done2 = apply_ok(
        &mut s,
        &cb,
        ImportTarget::NewStack("Organic Chemistry".into()),
        4,
    );
    assert_eq!(done2.created, 1);
    assert!(vault.join("organic-chemistry-2/_stack.md").is_file());

    let done3 = apply_ok(&mut s, &nb, ImportTarget::ExistingStack("french".into()), 5);
    assert_eq!(done3.created, 1);
    assert!(vault.join("french/root-note.md").is_file());
}

#[test]
fn pdf_import_supports_existing_and_new_stacks_and_rejects_unstacked() {
    let (_d, mut s, src) = setup("pdf");
    let pdf = src.join("lecture.pdf");
    fs::write(&pdf, pdfmake::three()).unwrap();
    assert_eq!(detect_format(&pdf), Some(SourceFormat::Pdf));
    assert!(read_source(&pdf).unwrap_err().contains("PDF"));

    let (_info, rel1) = import_pdf_to(
        &mut s,
        &pdf,
        &ImportTarget::ExistingStack("french".into()),
        ids(6),
        1_800_000_000_000_000_000,
    )
    .unwrap();
    assert_eq!(rel1, "french/lecture.pdf");
    let rows = pdf_library(&s).unwrap();
    assert_eq!(
        (rows.len(), rows[0].path.as_str(), rows[0].pages),
        (1, "french/lecture.pdf", 3)
    );

    let (info2, rel2) = import_pdf_to(
        &mut s,
        &pdf,
        &ImportTarget::NewStack("Physics".into()),
        ids(7),
        1_800_000_000_000_000_000,
    )
    .unwrap();
    assert_eq!(rel2, "physics/lecture.pdf");
    assert!(
        info2
            .overview
            .stacks
            .iter()
            .any(|st| st.folder == "physics")
    );
    assert_eq!(pdf_library(&s).unwrap().len(), 2);

    for bad in [ImportTarget::VaultRootNotes, ImportTarget::FromSource] {
        assert!(matches!(
            import_pdf_to(&mut s, &pdf, &bad, ids(8), 1_000),
            Err(ServiceError::Invalid(_))
        ));
    }
}
