#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Highlights and notes beside a PDF: saved as text, searchable, followed when edited by hand.

#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::path::PathBuf;

use studyfrog_config::pdf_marks::{MarkKind, MarkRect, PdfMark};
use studyfrog_service::backup::{BackupOptions, create_backup};
use studyfrog_service::pdf_marks::{load_marks, mark_pages_now, save_marks};
use studyfrog_service::pdfs::{find_pdfs, index_pdfs};
use studyfrog_service::restore::restore_backup;
use studyfrog_service::session::{ServiceError, VaultSession, open_vault};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const LATER: i64 = i64::MAX / 2;

fn session(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-marks-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("bio")).unwrap();
    fs::write(dir.join("bio/three.pdf"), pdfmake::three()).unwrap();
    let (mut s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    index_pdfs(&mut s, LATER).unwrap();
    (Dir(dir), s)
}

fn mark(id: u32, page: usize, covered: &str, note: &str) -> PdfMark {
    PdfMark {
        id,
        page,
        kind: MarkKind::Highlight,
        color: "blue".into(),
        rects: vec![MarkRect {
            x: 72.0,
            y: 80.0,
            width: 60.0,
            height: 14.0,
        }],
        covered: covered.into(),
        note: note.into(),
    }
}

#[test]
fn marks_are_saved_next_to_the_pdf_loaded_again_and_found_by_the_search() {
    let (dir, mut s) = session("save");
    assert!(load_marks(&s, "bio/three.pdf").unwrap().is_empty());
    let marks = vec![mark(1, 1, "Beta", "remember the mitochondria")];
    save_marks(&mut s, "bio/three.pdf", &marks).unwrap();
    assert!(dir.0.join("bio/three.pdf.notes.toml").exists());
    assert_eq!(load_marks(&s, "bio/three.pdf").unwrap(), marks);
    let hits = find_pdfs(&s, "mitochondria", 10).unwrap();
    assert_eq!(
        (hits.len(), hits[0].annotation, hits[0].page),
        (1, true, Some(2))
    );
    save_marks(&mut s, "bio/three.pdf", &[]).unwrap();
    assert!(
        !dir.0.join("bio/three.pdf.notes.toml").exists(),
        "no marks, no file"
    );
    assert!(find_pdfs(&s, "mitochondria", 10).unwrap().is_empty());
}

#[test]
fn a_notes_file_edited_by_hand_is_picked_up_by_the_next_pass() {
    let (dir, mut s) = session("hand");
    let text = "[[mark]]\nid = 1\npage = 3\nkind = \"note\"\ncolor = \"pink\"\nrects = [[10, 10, 18, 18]]\nnote = \"written outside\"\n";
    fs::write(dir.0.join("bio/three.pdf.notes.toml"), text).unwrap();
    assert!(index_pdfs(&mut s, LATER).unwrap());
    assert_eq!(
        find_pdfs(&s, "written outside", 5).unwrap()[0].page,
        Some(3)
    );
    assert!(!index_pdfs(&mut s, LATER).unwrap(), "unchanged afterwards");
}

#[test]
fn a_damaged_notes_file_is_reported_with_its_name_and_other_paths_are_refused() {
    let (dir, s) = session("bad");
    fs::write(dir.0.join("bio/three.pdf.notes.toml"), "[[mark]]\nid = 1\n").unwrap();
    let e = load_marks(&s, "bio/three.pdf").unwrap_err();
    assert!(
        matches!(&e, ServiceError::Invalid(m) if m.contains("three.pdf.notes.toml")),
        "{e:?}"
    );
    for bad in ["../x.pdf", "/etc/x.pdf", "bio/a.txt"] {
        assert!(load_marks(&s, bad).is_err(), "{bad}");
    }
}

#[test]
fn marks_follow_their_text_when_the_pdf_changes() {
    let (_d, s) = session("moved");
    let marks = vec![
        mark(1, 0, "Gamma", ""),
        mark(2, 1, "Beta", ""),
        mark(3, 1, "vanished text", ""),
    ];
    let now = mark_pages_now(&s, "bio/three.pdf", &marks).unwrap();
    assert_eq!(now, [Some(2), Some(1), None]);
}

#[test]
fn a_backup_carries_the_pdf_and_its_notes_and_a_restore_brings_them_back() {
    let (dir, mut s) = session("backup");
    save_marks(&mut s, "bio/three.pdf", &[mark(1, 0, "Alpha", "keep")]).unwrap();
    let dest = dir
        .0
        .join("..")
        .join(format!("studyfrog-marks-backups-{}", std::process::id()));
    let opts = BackupOptions {
        config_dir: None,
        include_keys: false,
        now_ms: 1_800_000_000_000,
        app_version: "0.1.0",
    };
    let report = create_backup(&s, &dest, opts).unwrap();
    let target = dest.join("restored");
    restore_backup(&report.path, &target, None).unwrap();
    assert!(target.join("bio/three.pdf").exists());
    let notes = fs::read_to_string(target.join("bio/three.pdf.notes.toml")).unwrap();
    assert!(notes.contains("note = \"keep\""));
    let _ = fs::remove_dir_all(&dest);
}
