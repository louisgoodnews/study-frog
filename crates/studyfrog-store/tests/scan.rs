#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Vault scanning, stack ownership and library building.

mod common;

use common::vault::{card, note, stack, write};
use studyfrog_core::arena::{arena_get, arena_len};
use studyfrog_core::uid::Uid;
use studyfrog_md::error::ParseErrorKind;
use studyfrog_store::library::{INBOX_TITLE, build_library};
use studyfrog_store::scan::scan_vault;

#[test]
fn stacks_own_the_nearest_folder_and_hidden_files_are_skipped() {
    let dir = common::temp_dir("scan");
    let r = &dir.path;
    write(r, "french/_stack.md", &stack(1, "French"));
    write(r, "french/paris.md", &card(2, "Paris", "Capital?", "Paris"));
    write(r, "french/grammar/_stack.md", &stack(3, "Grammar"));
    write(
        r,
        "french/grammar/verbs.md",
        &note(4, "Verbs", "See [[Paris]]\n"),
    );
    write(r, "loose.md", &note(5, "Loose", "no stack\n"));
    write(r, ".obsidian/ignored.md", &note(6, "Ignored", ""));
    write(r, "readme.txt", "not markdown");

    let scan = scan_vault(r).unwrap();
    let rels: Vec<_> = scan
        .files
        .iter()
        .map(|f| (f.file.rel.as_str(), f.stack.as_deref()))
        .collect();
    assert_eq!(
        rels,
        [
            ("french/_stack.md", Some("french")),
            ("french/grammar/_stack.md", Some("french/grammar")),
            ("french/grammar/verbs.md", Some("french/grammar")),
            ("french/paris.md", Some("french")),
            ("loose.md", None),
        ]
    );

    let lib = build_library(&scan);
    let titles: Vec<_> = lib
        .stacks
        .items
        .iter()
        .map(|s| s.meta.title.as_str())
        .collect();
    assert_eq!(titles, ["French", "Grammar", INBOX_TITLE]);
    assert_eq!((arena_len(&lib.notes), arena_len(&lib.cards)), (2, 1));
    let loose = &lib
        .notes
        .items
        .iter()
        .find(|n| n.meta.uid == Uid(5))
        .unwrap();
    assert_eq!(
        arena_get(&lib.stacks, loose.stack).unwrap().meta.uid,
        Uid(0)
    );
}

#[test]
fn bad_files_are_reported_and_never_fatal() {
    let dir = common::temp_dir("bad");
    let r = &dir.path;
    write(r, "good.md", &note(1, "Good", ""));
    write(r, "plain.md", "no front matter at all");
    write(
        r,
        "nocard.md",
        &note(2, "x", "").replace("kind: note", "kind: card"),
    );
    std::fs::write(r.join("binary.md"), [0xff, 0xfe, 0x00]).unwrap();

    let scan = scan_vault(r).unwrap();
    let status: Vec<_> = scan
        .files
        .iter()
        .map(|f| (f.file.rel.as_str(), f.outcome.is_ok()))
        .collect();
    assert_eq!(
        status,
        [
            ("binary.md", false),
            ("good.md", true),
            ("nocard.md", false),
            ("plain.md", false)
        ]
    );
    let missing = scan
        .files
        .iter()
        .find(|f| f.file.rel == "nocard.md")
        .unwrap();
    assert!(matches!(
        missing.outcome.as_ref().unwrap_err().kind,
        ParseErrorKind::MissingSeparator
    ));
    assert_eq!(arena_len(&build_library(&scan).notes), 1);
}

#[test]
fn duplicate_ids_flag_every_file_involved() {
    let dir = common::temp_dir("dup");
    let r = &dir.path;
    write(r, "a.md", &note(9, "A", ""));
    write(r, "b.md", &note(9, "B", ""));
    write(r, "c.md", &note(10, "C", ""));
    let scan = scan_vault(r).unwrap();
    let ok: Vec<_> = scan.files.iter().map(|f| f.outcome.is_ok()).collect();
    assert_eq!(ok, [false, false, true]);
    let msg = scan.files[0].outcome.as_ref().unwrap_err().to_string();
    assert!(msg.contains("a.md") && msg.contains("b.md"), "{msg}");
}
