#![allow(dead_code, clippy::expect_used, clippy::unwrap_used)]

//! Helpers to write small vaults for tests.

use std::fs;
use std::path::Path;

use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;

/// Write `rel` under `root`, creating folders.
pub fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, text).expect("write file");
}

pub fn note(id: u128, title: &str, body: &str) -> String {
    format!(
        "---\nid: {}\nkind: note\ntitle: {title}\n---\n{body}",
        format_ulid(Uid(id))
    )
}

pub fn card(id: u128, title: &str, front: &str, back: &str) -> String {
    format!(
        "---\nid: {}\nkind: card\ntitle: {title}\n---\n{front}\n???\n{back}\n",
        format_ulid(Uid(id))
    )
}

pub fn stack(id: u128, title: &str) -> String {
    format!(
        "---\nid: {}\nkind: stack\ntitle: {title}\n---\n",
        format_ulid(Uid(id))
    )
}
