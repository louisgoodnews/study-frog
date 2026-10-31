//! Walking the vault. The only module that lists directories.

use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::error::{StoreError, io_err};

/// A file found in the vault, with the stat data used for change detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultFile {
    /// Vault-relative path with `/` separators.
    pub rel: String,
    /// Absolute path.
    pub abs: PathBuf,
    /// Size in bytes.
    pub size: u64,
    /// Modification time, nanoseconds since the Unix epoch.
    pub mtime_ns: i64,
}

/// Every `.md` file under `root`, sorted by relative path. Hidden entries (leading `.`) and
/// symlinks are skipped.
pub fn list_markdown(root: &Path) -> Result<Vec<VaultFile>, StoreError> {
    list_files(root, "md")
}

/// Every `.pdf` file under `root`, sorted by relative path, with the same skips.
pub fn list_pdfs(root: &Path) -> Result<Vec<VaultFile>, StoreError> {
    list_files(root, "pdf")
}

fn list_files(root: &Path, ext: &str) -> Result<Vec<VaultFile>, StoreError> {
    let mut found = Vec::new();
    walk(root, root, ext, &mut found)?;
    found.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(found)
}

fn walk(root: &Path, dir: &Path, ext: &str, found: &mut Vec<VaultFile>) -> Result<(), StoreError> {
    for entry in fs::read_dir(dir).map_err(|e| io_err(dir, &e))? {
        let entry = entry.map_err(|e| io_err(dir, &e))?;
        visit(root, &entry, ext, found)?;
    }
    Ok(())
}

fn visit(
    root: &Path,
    entry: &DirEntry,
    ext: &str,
    found: &mut Vec<VaultFile>,
) -> Result<(), StoreError> {
    let path = entry.path();
    if entry.file_name().to_string_lossy().starts_with('.') {
        return Ok(());
    }
    let kind = entry.file_type().map_err(|e| io_err(&path, &e))?;
    if kind.is_dir() {
        return walk(root, &path, ext, found);
    }
    if kind.is_file()
        && path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
    {
        found.push(describe(root, &path, entry)?);
    }
    Ok(())
}

fn describe(root: &Path, path: &Path, entry: &DirEntry) -> Result<VaultFile, StoreError> {
    let meta = entry.metadata().map_err(|e| io_err(path, &e))?;
    let mtime_ns = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX));
    let rel = path.strip_prefix(root).unwrap_or(path);
    let rel = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    Ok(VaultFile {
        rel,
        abs: path.to_path_buf(),
        size: meta.len(),
        mtime_ns,
    })
}
