#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Obsidian importer: folders, notes, front matter, tags, pictures, unsupported syntax counted, hostile folders.

use std::fs;

use studyfrog_service::create::IdSource;
use studyfrog_service::session::open_vault;
use studyfrog_service::xfer_import::{Policy, apply_import, counts, plan_import};
use studyfrog_service::xfer_obsidian::{from_entries, from_folder};

fn e(path: &str, text: &str) -> (String, Vec<u8>) {
    (path.to_owned(), text.as_bytes().to_vec())
}

fn vault_files() -> Vec<(String, Vec<u8>)> {
    vec![
        e("Root note.md", "Just text with #loose-tag here.\n"),
        e(
            "Biology/Cells.md",
            "---\ntitle: Cell notes\ntags: [bio, \"exam\"]\naliases:\n  - cells\ncssclass: wide\nlinks:\n  - a\n---\nSee [[Mitosis]] and ![[cell.png]] and ![alt](assets/organelle.jpg).\n> [!note] A callout\n==marked== text %%hidden%%\n![[Another note]]\n```dataview\nTABLE x\n```\nA line ^block-1\n",
        ),
        e(
            "Biology/Genetics/Mitosis.md",
            "---\ntags:\n  - bio\n  - genetics\n---\n# Heading\nBody\n",
        ),
        e("Biology/cell.png", "PNG"),
        e("Biology/assets/organelle.jpg", "JPG"),
        e(".obsidian/app.json", "{}"),
        e(".obsidian/notes.md", "ignored"),
        e("Unrelated.txt", "ignored"),
    ]
}

#[test]
fn folders_become_stacks_notes_keep_front_matter_links_tags_and_pictures() {
    let b = from_entries(&vault_files()).unwrap();
    let folders: Vec<&str> = b.stacks.iter().map(|s| s.folder.as_str()).collect();
    assert_eq!(folders, ["Obsidian", "Biology", "Biology/Genetics"]);
    assert_eq!(
        b.items.len(),
        3,
        "the .obsidian folder and other files are left out"
    );
    let cells = b.items.iter().find(|i| i.title == "Cell notes").unwrap();
    assert_eq!(cells.stack, "Biology");
    assert_eq!(cells.tags, ["bio", "exam"], "front-matter tags first");
    assert_eq!(cells.extra, [("cssclass".to_owned(), "wide".to_owned())]);
    assert!(cells.text.contains("[[Mitosis]]"), "wikilinks are kept");
    assert!(
        cells.text.contains("![](images/obsidian/cell.png)")
            && cells.text.contains("![](images/obsidian/organelle.jpg)")
    );
    let paths: Vec<&str> = b.files.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(
        paths,
        ["images/obsidian/cell.png", "images/obsidian/organelle.jpg"]
    );
    let mitosis = b.items.iter().find(|i| i.title == "Mitosis").unwrap();
    assert_eq!(mitosis.tags, ["bio", "genetics"]);
    let root = b.items.iter().find(|i| i.title == "Root note").unwrap();
    assert_eq!(
        (root.stack.as_str(), root.tags.as_slice()),
        ("Obsidian", ["loose-tag".to_owned()].as_slice())
    );
}

#[test]
fn syntax_that_is_not_supported_stays_as_text_and_is_counted_for_the_preview() {
    let b = from_entries(&vault_files()).unwrap();
    let all = b.warnings.join(" | ");
    for expect in [
        "1 callouts",
        "1 dataview blocks",
        "1 comments",
        "1 highlights",
        "1 block references",
        "1 embedded notes",
        "aliases",
        "links",
    ] {
        assert!(all.contains(expect), "missing {expect:?} in {all}");
    }
    let cells = b.items.iter().find(|i| i.title == "Cell notes").unwrap();
    assert!(
        cells.text.contains("> [!note] A callout")
            && cells.text.contains("==marked==")
            && cells.text.contains("![[Another note]]")
    );
}

#[test]
fn a_folder_on_disk_imports_into_a_vault_with_its_pictures_and_kept_front_matter() {
    let src = std::env::temp_dir().join(format!("studyfrog-obs-src-{}", std::process::id()));
    let dst = std::env::temp_dir().join(format!("studyfrog-obs-dst-{}", std::process::id()));
    for d in [&src, &dst] {
        let _ = fs::remove_dir_all(d);
        fs::create_dir_all(d).unwrap();
    }
    for (p, bytes) in vault_files() {
        let f = src.join(&p);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(f, bytes).unwrap();
    }
    let b = from_folder(&src).unwrap();
    assert_eq!(b.items.len(), 3);
    let (mut s, _) = open_vault(dst.to_str().unwrap()).unwrap();
    let plan = plan_import(&s, &b, None).unwrap();
    assert_eq!(counts(&plan), (3, 0, 0));
    assert_eq!(plan.new_stacks, 3);
    let ids = IdSource {
        now_ms: 1_800_000_000_000,
        random: [5; 10],
    };
    let done = apply_import(&mut s, &b, None, (Policy::Skip, false), ids).unwrap();
    assert_eq!(done.created, 3, "{:?}", done.notes);
    assert!(dst.join("images/obsidian/cell.png").is_file());
    let note = fs::read_to_string(dst.join("Biology/cell-notes.md")).unwrap();
    assert!(
        note.contains("cssclass: wide")
            && note.contains("title: Cell notes")
            && note.contains("[[Mitosis]]"),
        "{note}"
    );
    let again = plan_import(&s, &b, None).unwrap();
    assert_eq!(
        counts(&again).0,
        3,
        "notes have no front and back, so they are not matched by their text"
    );
    for d in [&src, &dst] {
        let _ = fs::remove_dir_all(d);
    }
}

#[test]
fn hostile_and_empty_folders_are_errors_or_safe() {
    assert!(from_entries(&[]).is_err());
    assert!(from_entries(&[e("a.txt", "x")]).is_err());
    let b = from_entries(&[
        ("../../evil.md".to_owned(), b"x".to_vec()),
        ("/abs/path.md".to_owned(), b"x".to_vec()),
        ("bin.md".to_owned(), vec![0xff, 0xfe, 0xfd]),
        e(
            "deep/"
                .repeat(300)
                .trim_end_matches('/')
                .to_owned()
                .as_str(),
            "x",
        ),
        e("ok/Note.md", "---\nunterminated: yes\nbody"),
    ])
    .unwrap();
    assert!(
        b.stacks
            .iter()
            .all(|s| !s.folder.contains("..") && !s.folder.starts_with('/')),
        "{:?}",
        b.stacks
    );
    assert!(b.warnings.iter().any(|w| w.contains("not valid UTF-8")));
    assert!(from_folder(std::path::Path::new("/nonexistent-folder-xyz")).is_err());
}
