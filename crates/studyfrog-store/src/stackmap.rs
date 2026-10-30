//! Which stack owns which file: the nearest ancestor folder containing `_stack.md`.

use std::collections::HashSet;

use crate::vault::VaultFile;

/// File name that turns a folder into a stack.
pub const STACK_FILE: &str = "_stack.md";

/// Folder part of a vault-relative path; the vault root is the empty string.
pub fn parent_folder(rel: &str) -> &str {
    rel.rsplit_once('/').map_or("", |(dir, _)| dir)
}

/// Folders that contain a `_stack.md`.
pub fn stack_folders(files: &[VaultFile]) -> HashSet<String> {
    files
        .iter()
        .filter(|f| f.rel.rsplit('/').next() == Some(STACK_FILE))
        .map(|f| parent_folder(&f.rel).to_owned())
        .collect()
}

/// The stack folder that owns `rel`, or `None` when it sits outside every stack.
pub fn owning_stack(rel: &str, folders: &HashSet<String>) -> Option<String> {
    let mut dir = parent_folder(rel);
    loop {
        if folders.contains(dir) {
            return Some(dir.to_owned());
        }
        if dir.is_empty() {
            return None;
        }
        dir = parent_folder(dir);
    }
}
