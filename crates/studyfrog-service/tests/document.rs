#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Loading and saving documents safely.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_service::document::{load_document, resolve_link, save_document};
use studyfrog_service::session::{ServiceError, open_vault};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const NOTE: &str = "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Alpha\ncolour: red   # keep me\nfavourite: [a, b]\n---\nHello [[Beta]]\n";

fn vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-doc-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("s")).unwrap();
    fs::write(
        dir.join("s/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    )
    .unwrap();
    fs::write(dir.join("s/alpha.md"), NOTE).unwrap();
    fs::write(
        dir.join("s/beta.md"),
        "---\nid: 0000000000000000000000000B\nkind: note\ntitle: Beta\n---\n",
    )
    .unwrap();
    Dir(dir)
}

fn edited(doc: &Doc) -> Doc {
    let Doc::Note(n) = doc else {
        panic!("not a note")
    };
    let mut n = n.clone();
    n.meta.title = "Alpha (edited)".into();
    n.body = "Edited body [[Beta]]\n".into();
    Doc::Note(n)
}

#[test]
fn a_save_changes_the_file_keeps_unknown_keys_and_refreshes_the_index() {
    let d = vault("save");
    let (mut session, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let loaded = load_document(&session, Uid(0xA)).unwrap();
    assert_eq!(loaded.path, "s/alpha.md");

    let now = UnixMs(1_800_000_000_000);
    let saved = save_document(
        &mut session,
        &loaded.path,
        &edited(&loaded.doc),
        loaded.hash,
        now,
    )
    .unwrap();
    let text = fs::read_to_string(d.0.join("s/alpha.md")).unwrap();
    assert!(
        text.contains("title: Alpha (edited)") && text.contains("Edited body"),
        "{text}"
    );
    assert!(
        text.contains("colour: red   # keep me\nfavourite: [a, b]\n"),
        "unknown keys survive verbatim: {text}"
    );
    assert!(
        text.contains("updated: \"2027-01-15T08:00:00.000Z\""),
        "the updated time is set: {text}"
    );
    assert!(saved.info.overview.problems.is_empty());
    let again = load_document(&session, Uid(0xA)).unwrap();
    assert_eq!(
        again.hash, saved.hash,
        "the returned fingerprint matches the file"
    );
    assert!(!d.0.join("s/alpha.md.tmp").exists());
    assert_eq!(
        resolve_link(&session, "alpha (EDITED)").unwrap(),
        Some(Uid(0xA)),
        "the index sees the new title"
    );
}

#[test]
fn a_file_changed_elsewhere_is_never_overwritten() {
    let d = vault("conflict");
    let (mut session, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let loaded = load_document(&session, Uid(0xA)).unwrap();
    fs::write(
        d.0.join("s/alpha.md"),
        NOTE.replace("Hello", "Changed in another app"),
    )
    .unwrap();

    let err = save_document(
        &mut session,
        &loaded.path,
        &edited(&loaded.doc),
        loaded.hash,
        UnixMs(1),
    )
    .unwrap_err();
    assert!(
        matches!(err, ServiceError::Conflict(ref m) if m.contains("outside StudyFrog")),
        "{err:?}"
    );
    assert!(
        fs::read_to_string(d.0.join("s/alpha.md"))
            .unwrap()
            .contains("Changed in another app")
    );

    let reloaded = load_document(&session, Uid(0xA)).unwrap();
    assert!(
        save_document(
            &mut session,
            &reloaded.path,
            &edited(&reloaded.doc),
            reloaded.hash,
            UnixMs(1)
        )
        .is_ok(),
        "reloading resolves it"
    );
}

#[test]
fn unresolvable_and_unreadable_cases_are_reported() {
    let d = vault("errors");
    let (session, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    assert!(matches!(
        load_document(&session, Uid(999)),
        Err(ServiceError::Invalid(_))
    ));
    assert_eq!(resolve_link(&session, "nothing here").unwrap(), None);
    assert_eq!(
        resolve_link(&session, "  beta ").unwrap(),
        Some(Uid(0xB)),
        "case and spaces do not matter"
    );
    assert_eq!(
        resolve_link(&session, "S").unwrap(),
        None,
        "stacks are not link targets"
    );
}
