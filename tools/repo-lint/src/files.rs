//! Recursive file discovery.

use std::fs;
use std::path::{Path, PathBuf};

const SKIPPED_DIRS: [&str; 3] = ["target", ".git", "node_modules"];

/// All files under `root` with the given extension, skipping build and VCS directories.
pub fn find_files(root: &Path, extension: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    walk(root, extension, &mut found);
    found.sort();
    found
}

fn walk(dir: &Path, extension: &str, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for path in entries.flatten().map(|e| e.path()) {
        visit_path(&path, extension, found);
    }
}

fn visit_path(path: &Path, extension: &str, found: &mut Vec<PathBuf>) {
    if path.is_dir() {
        if !is_skipped(path) {
            walk(path, extension, found);
        }
    } else if path.extension().is_some_and(|e| e == extension) {
        found.push(path.to_path_buf());
    }
}

fn is_skipped(dir: &Path) -> bool {
    dir.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| SKIPPED_DIRS.contains(&n))
}
