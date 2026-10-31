#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Adding and renaming PDFs in the vault.

#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::path::PathBuf;

use studyfrog_service::create::IdSource;
use studyfrog_service::pdf_library::{add_pdf, rename_pdf};
use studyfrog_service::pdf_notes::{NoteRequest, note_from_pdf};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::pdf_notes::items_made_from;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn session(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-plib-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("bio")).unwrap();
    fs::write(
        dir.join("bio/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: Bio\n---\n",
    )
    .unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

#[test]
fn a_pdf_is_copied_into_the_stack_under_a_safe_name_and_never_over_another_file() {
    let (dir, s) = session("add");
    let outside = dir
        .0
        .join("..")
        .join(format!("studyfrog-outside-{}", std::process::id()));
    fs::create_dir_all(&outside).unwrap();
    let src = outside.join("My Lecture (2).pdf");
    fs::write(&src, pdfmake::three()).unwrap();
    let first = add_pdf(&s, &src, "bio").unwrap();
    let second = add_pdf(&s, &src, "bio").unwrap();
    assert_eq!(
        (first.as_str(), second.as_str()),
        ("bio/My-Lecture-2.pdf", "bio/My-Lecture-2-2.pdf")
    );
    assert!(
        dir.0.join(&first).is_file() && src.is_file(),
        "a copy, the original stays"
    );
    let text = outside.join("notes.pdf");
    fs::write(&text, "just words").unwrap();
    assert!(add_pdf(&s, &text, "bio").is_err(), "not a PDF inside");
    assert!(add_pdf(&s, &src, "nowhere").is_err(), "not a stack");
    assert!(add_pdf(&s, &src, "").is_err());
    assert!(add_pdf(&s, &outside.join("a.txt"), "bio").is_err());
    let _ = fs::remove_dir_all(outside);
}

#[test]
fn renaming_moves_the_notes_file_and_rewrites_sources_and_links() {
    let (dir, mut s) = session("rename");
    fs::write(dir.0.join("bio/old.pdf"), pdfmake::three()).unwrap();
    fs::write(
        dir.0.join("bio/old.pdf.notes.toml"),
        "[[mark]]\nid = 1\npage = 1\nkind = \"note\"\ncolor = \"blue\"\nrects = [[1, 1, 16, 16]]\n",
    )
    .unwrap();
    studyfrog_service::pdfs::index_pdfs_fully(&mut s, i64::MAX / 2).unwrap();
    let ids = IdSource {
        now_ms: 1_800_000_000_000,
        random: [1; 10],
    };
    let req = NoteRequest {
        pdf: "bio/old.pdf",
        first: 0,
        last: 0,
        selection: None,
        folder: "bio",
        title: "From old",
        pictures: &[],
    };
    let (_, made) = note_from_pdf(&mut s, &req, ids).unwrap();
    fs::write(dir.0.join("bio/linker.md"), "---\nid: 0000000000000000000000000B\nkind: note\ntitle: Linker\n---\nSee [page](bio/old.pdf#page=2).\n").unwrap();
    let (new, changed) = rename_pdf(&s, "bio/old.pdf", "new name.pdf").unwrap();
    assert_eq!((new.as_str(), changed), ("bio/new name.pdf", 2));
    assert!(
        !dir.0.join("bio/old.pdf").exists() && dir.0.join("bio/new name.pdf.notes.toml").is_file()
    );
    assert!(
        fs::read_to_string(dir.0.join("bio/linker.md"))
            .unwrap()
            .contains("(bio/new name.pdf#page=2)")
    );
    assert!(
        fs::read_to_string(dir.0.join(&made.path))
            .unwrap()
            .contains("bio/new name.pdf#page=1")
    );
    let (mut s, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    assert_eq!(
        items_made_from(&s.conn, "bio/new name.pdf", None)
            .unwrap()
            .len(),
        1
    );
    studyfrog_service::pdfs::index_pdfs_fully(&mut s, i64::MAX / 2).unwrap();
    assert!(rename_pdf(&s, "bio/new name.pdf", "../x.pdf").is_err());
    assert!(rename_pdf(&s, "bio/new name.pdf", "x.txt").is_err());
    fs::write(dir.0.join("bio/taken.pdf"), pdfmake::three()).unwrap();
    assert!(rename_pdf(&s, "bio/new name.pdf", "taken.pdf").is_err());
}
