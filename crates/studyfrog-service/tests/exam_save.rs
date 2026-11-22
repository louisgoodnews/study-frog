#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Exam notes and saving (STORY-0186): a folder's notes become topic chunks; a generated exam is saved as a
//! note with its reference answers.

use std::fs;
use std::path::PathBuf;

use studyfrog_rehearsal::curriculum_build::{assemble, plan, templates};
use studyfrog_rehearsal::mock_exam::validate_mock_exam;
use studyfrog_service::exam_save::{exam_markdown, notes_chunks, save_exam};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::reindex::reindex;

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!(
        "studyfrog-exam-save-{label}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("bio")).unwrap();
    fs::write(
        dir.join("bio/_stack.md"),
        "---\nid: 00000000000000000000000002\nkind: stack\ntitle: Bio\n---\n",
    )
    .unwrap();
    fs::write(
        dir.join("bio/cells.md"),
        "---\nid: 00000000000000000000000003\nkind: note\ntitle: Cells\n---\nMitochondria: organelles that produce ATP for the cell.\n",
    )
    .unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

#[test]
fn a_folders_notes_become_topic_chunks_and_an_exam_is_saved_as_a_note() {
    let (d, mut s) = vault("save");
    reindex(&mut s.conn, &s.root).unwrap();
    let chunks = notes_chunks(&s, Some("bio")).unwrap();
    assert!(
        chunks
            .iter()
            .any(|c| c.topic == "Cells" && c.text.contains("Mitochondria")),
        "{chunks:?}"
    );

    let want = plan(3);
    let qs = templates(&chunks, want);
    let (exam, keys) = assemble("Bio Klausur", &qs);
    assert!(validate_mock_exam(&exam).is_ok());
    let md = exam_markdown(&exam, &keys);
    assert!(md.contains("## Reference answers") && md.contains("Mitochondria"));

    let path = save_exam(
        &mut s,
        Some("bio"),
        "Bio Klausur",
        &exam,
        &keys,
        1_700_000_000_000,
    )
    .unwrap();
    assert!(path.starts_with("bio/") && path.ends_with(".md"), "{path}");
    let saved = fs::read_to_string(d.0.join(&path)).unwrap();
    assert!(saved.contains("Bio Klausur") && saved.contains("Reference answers"));
}
