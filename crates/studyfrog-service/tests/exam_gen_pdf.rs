#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Exam generation from a lecture PDF (STORY-0186): a note is exported to a real PDF, its text is read back
//! and chunked, and an exam is made from it offline.

use std::fs;
use std::path::{Path, PathBuf};

use studyfrog_md::ulid::parse_ulid;
use studyfrog_rehearsal::mock_exam::validate_mock_exam;
use studyfrog_service::exam_gen::{chunks_of_pdf, generate};
use studyfrog_service::pdf_export::{ExportTarget, default_pdf_export_options, export_pdf};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::reindex::reindex;

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir =
        std::env::temp_dir().join(format!("studyfrog-exam-pdf-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn write(dir: &Path, rel: &str, content: &str) {
    fs::write(dir.join(rel), content).unwrap();
}

#[test]
fn an_exam_is_made_from_a_lecture_pdf_offline_and_is_valid() {
    let (d, mut s) = vault("lecture");
    let uid = parse_ulid("00000000000000000000000001").unwrap();
    let md = "---\nid: 00000000000000000000000001\nkind: note\ntitle: Gütekriterien\n---\n# Reliabilität\n\nReliabilität: consistency of measurement over repeated testing occasions.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\n# Normierung\n\nStandard scores place a result against the norm group mean and standard deviation.\n";
    write(&d.0, "lecture.md", md);
    reindex(&mut s.conn, &s.root).unwrap();

    let pdf = d.0.join("lecture.pdf");
    export_pdf(
        &s,
        &ExportTarget::SingleItem(uid),
        &default_pdf_export_options(),
        &pdf,
    )
    .unwrap();
    let bytes = fs::read(&pdf).unwrap();
    assert!(bytes.starts_with(b"%PDF"));

    let chunks = chunks_of_pdf(&bytes).expect("the PDF's text is read");
    let text: String = chunks
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        text.contains("Reliabilität") && text.contains("norm group"),
        "{text}"
    );

    let g = generate(&chunks, 4, "Klausur aus dem Skript", None);
    assert!(validate_mock_exam(&g.exam).is_ok());
    assert!(!g.keys.is_empty());
}

#[test]
fn slides_in_the_vault_are_read_and_paths_outside_it_are_refused() {
    use studyfrog_service::exam_save::slide_chunks;
    let (d, mut s) = vault("slides");
    let uid = parse_ulid("00000000000000000000000002").unwrap();
    let md = "---\nid: 00000000000000000000000002\nkind: note\ntitle: Folien\n---\n# Normierung\n\nStandard scores place a result against the norm group mean and standard deviation.\n";
    write(&d.0, "folien.md", md);
    reindex(&mut s.conn, &s.root).unwrap();
    export_pdf(
        &s,
        &ExportTarget::SingleItem(uid),
        &default_pdf_export_options(),
        &d.0.join("folien.pdf"),
    )
    .unwrap();

    let chunks = slide_chunks(&s, "folien.pdf").expect("the slides are read");
    let text: String = chunks.iter().map(|c| c.text.as_str()).collect();
    assert!(text.contains("norm group"), "{text}");
    let exam = generate(&chunks, 3, "Klausur aus den Folien", None);
    assert!(validate_mock_exam(&exam.exam).is_ok());

    assert!(slide_chunks(&s, "../folien.pdf").is_err());
    assert!(slide_chunks(&s, "/etc/folien.pdf").is_err());
    assert!(slide_chunks(&s, "folien.md").is_err());

    studyfrog_service::pdfs::index_pdfs_fully(&mut s, i64::MAX / 2).expect("indexed");
    let listed = studyfrog_service::pdfs::pdf_library(&s).expect("the library");
    assert!(
        listed
            .iter()
            .any(|r| r.path == "folien.pdf" && r.error.is_none()),
        "{listed:?}"
    );
}
