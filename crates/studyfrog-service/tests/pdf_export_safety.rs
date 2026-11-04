#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for PDF export safety: hostile paths, cancellation, progress reporting, missing images, and wikilinks.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use studyfrog_md::ulid::parse_ulid;
use studyfrog_pdf::text::pdf_all_text;
use studyfrog_service::pdf_export::{
    ExportProgress, ExportTarget, PdfExportError, default_pdf_export_options, execute_pdf_export,
    export_pdf,
};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::reindex::reindex;

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn pdf_text(bytes: Vec<u8>) -> String {
    let pages = pdf_all_text(&Arc::new(bytes)).unwrap();
    pages
        .into_iter()
        .map(|p| p.text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn test_vault(label: &str) -> (Dir, VaultSession) {
    let dir =
        std::env::temp_dir().join(format!("studyfrog-pdf-safe-{label}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn write_file(dir: &Path, rel: &str, content: &str) {
    let file = dir.join(rel);
    if let Some(p) = file.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(&file, content).unwrap();
}

#[test]
fn hostile_destination_paths_are_refused_leaving_no_files() {
    let (d, mut s) = test_vault("hostile_paths");
    let uid = parse_ulid("00000000000000000000000030").unwrap();
    write_file(
        &d.0,
        "doc.md",
        "---\nid: 00000000000000000000000030\nkind: note\ntitle: Doc\n---\nHello.\n",
    );
    reindex(&mut s.conn, &s.root).unwrap();

    let opts = default_pdf_export_options();
    let target = ExportTarget::SingleItem(uid);

    let not_pdf = d.0.join("bad.txt");
    assert!(export_pdf(&s, &target, &opts, &not_pdf).is_err());
    assert!(!not_pdf.exists());

    let null_byte = d.0.join("bad\0name.pdf");
    assert!(export_pdf(&s, &target, &opts, &null_byte).is_err());

    let reserved = d.0.join("nul.pdf");
    assert!(export_pdf(&s, &target, &opts, &reserved).is_err());

    let inside_index = d.0.join(".studyfrog").join("out.pdf");
    assert!(export_pdf(&s, &target, &opts, &inside_index).is_err());
    assert!(!inside_index.exists());
}

#[test]
fn export_cancellation_and_progress_reporting() {
    let (d, mut s) = test_vault("cancel_prog");
    let uid1 = parse_ulid("00000000000000000000000031").unwrap();
    let uid2 = parse_ulid("00000000000000000000000032").unwrap();
    write_file(
        &d.0,
        "n1.md",
        "---\nid: 00000000000000000000000031\nkind: note\ntitle: Note 1\n---\nFirst\n",
    );
    write_file(
        &d.0,
        "n2.md",
        "---\nid: 00000000000000000000000032\nkind: note\ntitle: Note 2\n---\nSecond\n",
    );
    reindex(&mut s.conn, &s.root).unwrap();

    let target = ExportTarget::Selection(vec![uid1, uid2]);
    let opts = default_pdf_export_options();
    let dest = d.0.join("cancelled.pdf");

    let cancel = AtomicBool::new(true);
    let mut progress_calls = Vec::new();
    let err = execute_pdf_export(
        &s,
        &target,
        &opts,
        &dest,
        Some(&cancel),
        Some(&mut |p: ExportProgress| progress_calls.push(p)),
    )
    .unwrap_err();

    assert_eq!(err, PdfExportError::Cancelled);
    assert!(!dest.exists(), "failed or cancelled export leaves no file");

    cancel.store(false, Ordering::Relaxed);
    progress_calls.clear();
    let success = execute_pdf_export(
        &s,
        &target,
        &opts,
        &dest,
        Some(&cancel),
        Some(&mut |p: ExportProgress| progress_calls.push(p)),
    )
    .unwrap();

    assert!(dest.exists());
    assert_eq!(success.item_count, 2);
    assert!(!progress_calls.is_empty());
    assert_eq!(progress_calls.last().map(|p| p.current), Some(2));
}

#[test]
fn missing_picture_is_labelled_gap_and_reported_without_aborting() {
    let (d, mut s) = test_vault("missing_img");
    let uid = parse_ulid("00000000000000000000000040").unwrap();
    write_file(&d.0, "assets/existing.png", "dummy png bytes");
    let md = "---\nid: 00000000000000000000000040\nkind: note\ntitle: Photo Gallery\n---\n![Valid](assets/existing.png)\n\n![Missing diagram](assets/ghost.png)\n";
    write_file(&d.0, "gallery.md", md);
    reindex(&mut s.conn, &s.root).unwrap();

    let dest = d.0.join("gallery.pdf");
    let outcome = export_pdf(
        &s,
        &ExportTarget::SingleItem(uid),
        &default_pdf_export_options(),
        &dest,
    )
    .unwrap();
    assert!(dest.exists());
    assert!(
        !outcome.warnings.is_empty(),
        "missing picture is reported in warnings"
    );
    assert!(outcome.warnings.iter().any(|w| w.contains("ghost.png")));

    let text = pdf_text(fs::read(&dest).unwrap());
    assert!(text.contains("[Missing picture: Missing diagram]"));
}

#[test]
fn wikilinks_become_linked_item_title() {
    let (d, mut s) = test_vault("wikilinks");
    let note_a_uid = parse_ulid("00000000000000000000000051").unwrap();
    let note_b_uid = parse_ulid("00000000000000000000000052").unwrap();
    let _external_uid = parse_ulid("00000000000000000000000053").unwrap();

    write_file(
        &d.0,
        "note_a.md",
        "---\nid: 00000000000000000000000051\nkind: note\ntitle: Alpha Note\n---\nSee [[Beta Note]] and external [[Gamma Vault Note]].\n",
    );
    write_file(
        &d.0,
        "note_b.md",
        "---\nid: 00000000000000000000000052\nkind: note\ntitle: Beta Note\n---\nI am Beta.\n",
    );
    write_file(
        &d.0,
        "note_gamma.md",
        "---\nid: 00000000000000000000000053\nkind: note\ntitle: Gamma Vault Note\n---\nExternal note.\n",
    );
    reindex(&mut s.conn, &s.root).unwrap();

    let dest = d.0.join("wikilinks_test.pdf");
    let selection = ExportTarget::Selection(vec![note_a_uid, note_b_uid]);
    let outcome = export_pdf(&s, &selection, &default_pdf_export_options(), &dest).unwrap();
    assert_eq!(outcome.item_count, 2);

    let text = pdf_text(fs::read(&dest).unwrap());
    assert!(text.contains("Alpha Note"));
    assert!(text.contains("Beta Note"));
    assert!(text.contains("Gamma Vault Note"));

    let doc = lopdf::Document::load(&dest).unwrap();
    assert!(!doc.get_pages().is_empty());
}
