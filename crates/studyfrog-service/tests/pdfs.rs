#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! PDF files in the vault are indexed once, found by title and text, and followed on change.

#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::path::PathBuf;

use studyfrog_service::pdfs::{FILES_PER_PASS, find_pdfs, index_pdfs, pdf_library};
use studyfrog_service::session::open_vault;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const LATER: i64 = i64::MAX / 2;

fn vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-pdfs-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("bio/deep")).unwrap();
    fs::write(dir.join("bio/two.pdf"), pdfmake::three()).unwrap();
    let page = vec![("Mitochondria make energy", 72.0, 700.0)];
    fs::write(
        dir.join("bio/deep/Cells.pdf"),
        pdfmake::make(&[page], Some("Cell biology")),
    )
    .unwrap();
    fs::write(dir.join("bio/broken.pdf"), b"this is not a pdf").unwrap();
    Dir(dir)
}

#[test]
fn every_pdf_is_indexed_with_title_pages_and_text_and_broken_ones_with_a_reason() {
    let dir = vault("index");
    let (mut session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    assert!(index_pdfs(&mut session, LATER).unwrap());
    let lib = pdf_library(&session).unwrap();
    let by = |p: &str| lib.iter().find(|r| r.path == p).unwrap().clone();
    assert_eq!(lib.len(), 3);
    assert_eq!(
        (
            by("bio/deep/Cells.pdf").title.as_str(),
            by("bio/deep/Cells.pdf").pages
        ),
        ("Cell biology", 1)
    );
    assert_eq!(by("bio/two.pdf").pages, 3);
    assert!(by("bio/broken.pdf").error.is_some());
    assert_eq!(
        by("bio/broken.pdf").title,
        "broken",
        "falls back to the file name"
    );
    let hits = find_pdfs(&session, "mitochondria", 10).unwrap();
    assert_eq!(
        (hits[0].path.as_str(), hits[0].page),
        ("bio/deep/Cells.pdf", Some(1))
    );
    let hits = find_pdfs(&session, "cell bio", 10).unwrap();
    assert_eq!(hits[0].page, None, "title hit");
}

#[test]
fn unchanged_files_are_not_read_again_and_changes_and_deletions_are_followed() {
    let dir = vault("change");
    let (mut session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    index_pdfs(&mut session, LATER).unwrap();
    assert!(
        !index_pdfs(&mut session, LATER).unwrap(),
        "second pass finds nothing to do"
    );
    let page = vec![("Ribosomes build proteins", 72.0, 700.0)];
    fs::write(
        dir.0.join("bio/deep/Cells.pdf"),
        pdfmake::make(&[page], Some("Cell biology")),
    )
    .unwrap();
    fs::remove_file(dir.0.join("bio/two.pdf")).unwrap();
    assert!(index_pdfs(&mut session, LATER).unwrap());
    assert!(find_pdfs(&session, "mitochondria", 10).unwrap().is_empty());
    assert_eq!(find_pdfs(&session, "ribosomes", 10).unwrap().len(), 1);
    assert_eq!(pdf_library(&session).unwrap().len(), 2);
}

#[test]
fn a_big_first_scan_is_spread_over_passes_and_a_fresh_file_waits_to_settle() {
    let dir = vault("passes");
    for i in 0..FILES_PER_PASS + 2 {
        fs::write(dir.0.join(format!("extra{i}.pdf")), pdfmake::three()).unwrap();
    }
    let (mut session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    assert!(
        !index_pdfs(&mut session, 0).unwrap(),
        "just written files are left to settle"
    );
    index_pdfs(&mut session, LATER).unwrap();
    assert_eq!(pdf_library(&session).unwrap().len(), FILES_PER_PASS);
    assert_eq!(session.pdf_pending, 5);
    while session.pdf_pending > 0 {
        index_pdfs(&mut session, LATER).unwrap();
    }
    index_pdfs(&mut session, LATER).unwrap();
    assert_eq!(pdf_library(&session).unwrap().len(), FILES_PER_PASS + 5);
}
