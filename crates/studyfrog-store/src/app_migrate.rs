//! Moving into the application home: files of the old config directory are copied across, never
//! moved or deleted, so the old version keeps working next to the new one.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::{StoreError, io_err};

/// The file that holds the provider keys.
const SECRETS_FILE: &str = "secrets.toml";

/// Copy every file under `legacy` to the same place under `config` unless one is already there.
/// Returns the files that were copied. A missing `legacy` directory means nothing to do.
pub fn migrate_legacy_config(legacy: &Path, config: &Path) -> Result<Vec<PathBuf>, StoreError> {
    let mut copied = Vec::new();
    copy_missing(legacy, config, &mut copied)?;
    secure_secrets(config)?;
    Ok(copied)
}

fn copy_missing(from: &Path, to: &Path, copied: &mut Vec<PathBuf>) -> Result<(), StoreError> {
    let entries = match fs::read_dir(from) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(io_err(from, &e)),
    };
    for entry in entries {
        let entry = entry.map_err(|e| io_err(from, &e))?;
        copy_entry(&entry, to, copied)?;
    }
    Ok(())
}

fn copy_entry(
    entry: &fs::DirEntry,
    to: &Path,
    copied: &mut Vec<PathBuf>,
) -> Result<(), StoreError> {
    let (src, dest) = (entry.path(), to.join(entry.file_name()));
    let kind = entry.file_type().map_err(|e| io_err(&src, &e))?;
    if kind.is_dir() {
        return copy_missing(&src, &dest, copied);
    }
    if !kind.is_file() || dest.exists() {
        return Ok(());
    }
    fs::create_dir_all(to).map_err(|e| io_err(to, &e))?;
    fs::copy(&src, &dest).map_err(|e| io_err(&dest, &e))?;
    copied.push(dest);
    Ok(())
}

/// Make the provider keys readable by the owner only. A missing file is fine.
#[cfg(unix)]
pub fn secure_secrets(config: &Path) -> Result<(), StoreError> {
    use std::os::unix::fs::PermissionsExt;
    let path = config.join(SECRETS_FILE);
    match fs::set_permissions(&path, fs::Permissions::from_mode(0o600)) {
        Err(e) if e.kind() != ErrorKind::NotFound => Err(io_err(&path, &e)),
        _ => Ok(()),
    }
}

/// Make the provider keys readable by the owner only. Nothing to do off Unix.
#[cfg(not(unix))]
pub fn secure_secrets(_config: &Path) -> Result<(), StoreError> {
    Ok(())
}
