//! Restoring a backup: unpack it into a new, empty folder and never overwrite anything.

use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Component, Path, PathBuf};

use studyfrog_config::app::{APP_CONFIG_FILE, parse_app_settings, render_app_settings};
use studyfrog_config::backup::{MANIFEST_FILE, Manifest, parse_manifest};
use studyfrog_core::settings::with_opened_vault;

use crate::backup::{SECRETS_FILE, set_private};
use crate::session::ServiceError;
use crate::tar::{TarEntry, tar_copy, tar_next};

const VAULT_PREFIX: &str = "vault/";
const DATABASE_ENTRY: &str = "database/index.sqlite";
const SETTINGS_ENTRY: &str = "config/app.toml";
const KEYS_ENTRY: &str = "config/secrets.toml";
const INDEX_DIR: &str = ".studyfrog";

fn io_err(path: &Path, e: std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{}: {e}", path.display()))
}

/// Read what a backup holds without unpacking it.
pub fn read_manifest(archive: &Path) -> Result<Manifest, ServiceError> {
    let mut r = BufReader::new(File::open(archive).map_err(|e| io_err(archive, e))?);
    let corrupt = |why: String| {
        ServiceError::Invalid(format!(
            "{} is not a StudyFrog backup: {why}",
            archive.display()
        ))
    };
    let entry = tar_next(&mut r)
        .map_err(|e| corrupt(e.to_string()))?
        .ok_or_else(|| corrupt("it is empty".to_owned()))?;
    if entry.name != MANIFEST_FILE || entry.size > 64 * 1024 {
        return Err(corrupt("it does not start with a manifest".to_owned()));
    }
    let mut text = Vec::new();
    tar_copy(&mut r, &entry, &mut text).map_err(|e| corrupt(e.to_string()))?;
    let text =
        String::from_utf8(text).map_err(|_| corrupt("the manifest is not text".to_owned()))?;
    parse_manifest(&text)
        .map_err(|e| ServiceError::Invalid(studyfrog_config::error::describe_config_error(&e)))
}

/// A path inside the archive as a safe relative path: no `..`, no root, no empty parts.
fn safe_relative(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    let plain = path.components().all(|c| matches!(c, Component::Normal(_)));
    (plain && !name.is_empty() && !name.contains('\\')).then(|| path.to_owned())
}

/// What a restore did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreReport {
    /// The new vault folder.
    pub vault: PathBuf,
    /// Files restored (without the database and the settings).
    pub files: u32,
    /// The app settings were restored (the previous ones are kept as `app.toml.before-restore`).
    pub settings_restored: bool,
    /// The API keys were restored.
    pub keys_restored: bool,
}

fn extract(r: &mut impl std::io::Read, entry: &TarEntry, to: &Path) -> Result<(), ServiceError> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;
    }
    let mut f = File::create_new(to).map_err(|e| io_err(to, e))?;
    tar_copy(r, entry, &mut f).map_err(|e| io_err(to, e))
}

/// Unpack a backup into `target`, a folder that does not exist yet or is empty. With `config_dir` set,
/// the settings (and keys) in the backup replace the current ones, which are kept next to them.
pub fn restore_backup(
    archive: &Path,
    target: &Path,
    config_dir: Option<&Path>,
) -> Result<RestoreReport, ServiceError> {
    let manifest = read_manifest(archive)?;
    if target.exists()
        && fs::read_dir(target)
            .map_err(|e| io_err(target, e))?
            .next()
            .is_some()
    {
        return Err(ServiceError::Invalid(format!(
            "{} is not empty; choose a new or empty folder",
            target.display()
        )));
    }
    fs::create_dir_all(target).map_err(|e| io_err(target, e))?;
    let result = unpack(archive, target, config_dir, &manifest);
    if result.is_err() {
        let _ = fs::remove_dir_all(target);
    }
    result
}

fn unpack(
    archive: &Path,
    target: &Path,
    config_dir: Option<&Path>,
    manifest: &Manifest,
) -> Result<RestoreReport, ServiceError> {
    let mut r = BufReader::new(File::open(archive).map_err(|e| io_err(archive, e))?);
    let bad = |e: std::io::Error| ServiceError::Invalid(format!("the backup is damaged: {e}"));
    let mut report = RestoreReport {
        vault: target.to_owned(),
        files: 0,
        settings_restored: false,
        keys_restored: false,
    };
    let mut database = false;
    while let Some(entry) = tar_next(&mut r).map_err(bad)? {
        if entry.name == MANIFEST_FILE {
            tar_copy(&mut r, &entry, &mut std::io::sink()).map_err(bad)?;
        } else if entry.name == DATABASE_ENTRY {
            extract(&mut r, &entry, &target.join(INDEX_DIR).join("index.sqlite"))?;
            database = true;
        } else if let Some(rel) = entry.name.strip_prefix(VAULT_PREFIX) {
            let rel = safe_relative(rel).ok_or_else(|| {
                ServiceError::Invalid(format!("the backup holds an unsafe path: {}", entry.name))
            })?;
            extract(&mut r, &entry, &target.join(rel))?;
            report.files += 1;
        } else if entry.name == SETTINGS_ENTRY || entry.name == KEYS_ENTRY {
            restore_config_file(&mut r, &entry, config_dir, target, &mut report)?;
        } else {
            tar_copy(&mut r, &entry, &mut std::io::sink()).map_err(bad)?;
        }
    }
    if !database || report.files != manifest.files {
        return Err(ServiceError::Invalid("the backup is incomplete".to_owned()));
    }
    Ok(report)
}

fn restore_config_file(
    r: &mut impl std::io::Read,
    entry: &TarEntry,
    config_dir: Option<&Path>,
    target: &Path,
    report: &mut RestoreReport,
) -> Result<(), ServiceError> {
    let is_settings = entry.name == SETTINGS_ENTRY;
    let Some(dir) = config_dir else {
        return tar_copy(r, entry, &mut std::io::sink())
            .map_err(|e| ServiceError::Invalid(format!("the backup is damaged: {e}")));
    };
    let mut text = Vec::new();
    tar_copy(r, entry, &mut text)
        .map_err(|e| ServiceError::Invalid(format!("the backup is damaged: {e}")))?;
    let file = dir.join(if is_settings {
        APP_CONFIG_FILE
    } else {
        SECRETS_FILE
    });
    fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;
    if file.exists() {
        let kept = file.with_extension("toml.before-restore");
        fs::copy(&file, &kept).map_err(|e| io_err(&kept, e))?;
    }
    let content = if is_settings {
        settings_for(&text, target)?
    } else {
        text
    };
    fs::write(&file, content).map_err(|e| io_err(&file, e))?;
    if is_settings {
        report.settings_restored = true;
    } else {
        set_private(&file)?;
        report.keys_restored = true;
    }
    Ok(())
}

/// The settings of the backup, with the restored vault as the current one.
fn settings_for(text: &[u8], target: &Path) -> Result<Vec<u8>, ServiceError> {
    let text = String::from_utf8_lossy(text);
    let settings = parse_app_settings(&text).map_err(|e| {
        ServiceError::Invalid(format!(
            "the settings in the backup: {}",
            studyfrog_config::error::describe_config_error(&e)
        ))
    })?;
    Ok(
        render_app_settings(&with_opened_vault(&settings, &target.display().to_string()))
            .into_bytes(),
    )
}
