#![allow(dead_code, unused_imports, clippy::unwrap_used, clippy::expect_used)]
//! A scratch vault with one stack for the active-recall tests (EPIC-035).

use std::fs;
use std::path::PathBuf;

use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::create::IdSource;
use studyfrog_service::session::{VaultSession, open_vault};

/// A folder removed when dropped.
pub struct Dir(pub PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The id written as a ULID.
pub fn id(n: u128) -> String {
    format_ulid(Uid(n))
}

/// A vault with the stack `bio` (id 1) holding `files` (name, text).
pub fn vault(label: &str, files: &[(&str, String)]) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-recall-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("bio")).unwrap();
    let stack = format!("---\nid: {}\nkind: stack\ntitle: Biology\n---\n", id(1));
    fs::write(dir.join("bio/_stack.md"), stack).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    Dir(dir)
}

/// Open the vault.
pub fn open(dir: &Dir) -> VaultSession {
    open_vault(dir.0.to_str().unwrap()).unwrap().0
}

/// A note file.
pub fn note(n: u128, title: &str, body: &str) -> String {
    format!(
        "---\nid: {}\nkind: note\ntitle: {title}\ntags:\n  - cells\n---\n{body}",
        id(n)
    )
}

/// A card file.
pub fn card(n: u128, title: &str, front: &str, back: &str) -> String {
    format!(
        "---\nid: {}\nkind: card\ntitle: {title}\n---\n{front}\n???\n{back}\n",
        id(n)
    )
}

/// A fixed id source.
pub fn ids(now_ms: u64) -> IdSource {
    IdSource {
        now_ms,
        random: [7; 10],
    }
}

/// The text of a file of the vault.
pub fn read(dir: &Dir, rel: &str) -> String {
    fs::read_to_string(dir.0.join(rel)).unwrap()
}

/// The Markdown files of a folder of the vault, sorted.
pub fn files_in(dir: &Dir, folder: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir.0.join(folder))
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".md"))
        .collect();
    names.sort();
    names
}
