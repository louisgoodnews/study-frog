#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Opening a vault and keeping it current.

use std::fs;
use std::path::PathBuf;

use studyfrog_service::session::{ServiceError, open_vault, poll_vault};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> Dir {
    let dir =
        std::env::temp_dir().join(format!("studyfrog-session-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("s")).unwrap();
    fs::write(
        dir.join("s/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    )
    .unwrap();
    fs::write(
        dir.join("s/a.md"),
        "---\nid: 0000000000000000000000000A\nkind: note\ntitle: A\n---\n",
    )
    .unwrap();
    Dir(dir)
}

fn note(id: &str) -> String {
    format!("---\nid: {id}\nkind: note\ntitle: N{id}\n---\n")
}

#[test]
fn opening_creates_a_hidden_index_and_describes_the_vault() {
    let d = vault("open");
    let (_, info) = open_vault(d.0.to_str().unwrap()).unwrap();
    assert_eq!(info.overview.stacks.len(), 1);
    assert_eq!(
        (info.overview.stacks[0].notes, info.settings_error),
        (1, None)
    );
    assert!(d.0.join(".studyfrog/index.sqlite").exists());
    open_vault(d.0.to_str().unwrap()).unwrap(); // reopening an indexed vault works
    let (_, again) = open_vault(d.0.to_str().unwrap()).unwrap();
    assert_eq!(
        again.overview, info.overview,
        "the hidden index folder is not scanned as content"
    );
}

#[test]
fn a_path_that_is_not_a_folder_is_refused() {
    let d = vault("notdir");
    let file = d.0.join("s/a.md");
    assert!(matches!(
        open_vault(file.to_str().unwrap()),
        Err(ServiceError::NotAFolder(_))
    ));
    assert!(matches!(
        open_vault("/definitely/not/here"),
        Err(ServiceError::NotAFolder(_))
    ));
}

#[test]
fn polling_reports_changes_once_and_waits_for_files_to_settle() {
    let d = vault("poll");
    let (mut session, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let later = i64::MAX / 2;
    assert!(
        poll_vault(&mut session, later).unwrap().is_none(),
        "nothing changed since opening"
    );

    fs::write(d.0.join("s/b.md"), note("0000000000000000000000000B")).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64;
    assert!(
        poll_vault(&mut session, now).unwrap().is_none(),
        "just written: settling"
    );
    let info = poll_vault(&mut session, later).unwrap().unwrap();
    assert_eq!(info.overview.stacks[0].notes, 2);
    assert!(
        poll_vault(&mut session, later).unwrap().is_none(),
        "reported once"
    );
}
