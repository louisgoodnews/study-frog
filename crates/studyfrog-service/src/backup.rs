//! Backups and restores: one tar archive with the vault's files, a consistent copy of its database
//! (reviews, runs, the board and the calendar live there), and optionally the app settings and keys.
//! A restore goes into a new, empty folder and never overwrites anything.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use studyfrog_config::app::APP_CONFIG_FILE;
use studyfrog_config::backup::{BACKUP_FORMAT, MANIFEST_FILE, Manifest, render_manifest};

use crate::session::{ServiceError, VaultSession};
use crate::tar::{tar_bytes, tar_finish, tar_header, tar_padding};

const VAULT_PREFIX: &str = "vault/";
const DATABASE_ENTRY: &str = "database/index.sqlite";
const SETTINGS_ENTRY: &str = "config/app.toml";
const KEYS_ENTRY: &str = "config/secrets.toml";
const PROVIDERS_ENTRY: &str = "config/providers.toml";
/// The file of the API keys in the config folder.
pub const SECRETS_FILE: &str = "secrets.toml";
const INDEX_DIR: &str = ".studyfrog";

fn io_err(path: &Path, e: std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{}: {e}", path.display()))
}

/// What to put in a backup besides the vault.
#[derive(Debug, Clone, Copy)]
pub struct BackupOptions<'a> {
    /// The user's config folder (settings and keys), if there is one.
    pub config_dir: Option<&'a Path>,
    /// Include the API keys.
    pub include_keys: bool,
    /// The time to record, in Unix milliseconds.
    pub now_ms: i64,
    /// The program version to record.
    pub app_version: &'a str,
}

/// What a finished backup is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupReport {
    /// The archive.
    pub path: PathBuf,
    /// What it holds.
    pub manifest: Manifest,
    /// The archive's size in bytes.
    pub archive_bytes: u64,
}

/// Every file under `root` (relative, `/`-separated), skipping the index folder and links.
fn vault_files(root: &Path) -> Result<Vec<String>, ServiceError> {
    let mut out = Vec::new();
    let mut todo = vec![PathBuf::new()];
    while let Some(rel) = todo.pop() {
        let dir = root.join(&rel);
        for entry in fs::read_dir(&dir).map_err(|e| io_err(&dir, e))? {
            let entry = entry.map_err(|e| io_err(&dir, e))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let kind = entry.file_type().map_err(|e| io_err(&dir, e))?;
            let rel_path = rel.join(&name);
            if kind.is_dir() && !(rel.as_os_str().is_empty() && name == INDEX_DIR) {
                todo.push(rel_path);
            } else if kind.is_file() && !name.ends_with(".tmp") {
                out.push(rel_path.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out.sort();
    Ok(out)
}

fn archive_name(vault: &str, now_ms: i64) -> String {
    let day = chrono::DateTime::from_timestamp_millis(now_ms)
        .map_or_else(String::new, |t| t.format("%Y%m%d-%H%M%S").to_string());
    let safe: String = vault
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    format!("studyfrog-{safe}-{day}.tar")
}

fn add_file(w: &mut impl Write, entry: &str, path: &Path, mtime: u64) -> Result<u64, ServiceError> {
    let size = fs::metadata(path).map_err(|e| io_err(path, e))?.len();
    tar_header(w, entry, size, mtime).map_err(|e| io_err(path, e))?;
    let mut f = File::open(path).map_err(|e| io_err(path, e))?;
    let copied = std::io::copy(&mut f, w).map_err(|e| io_err(path, e))?;
    if copied != size {
        return Err(ServiceError::Io(format!(
            "{} changed while it was being copied; try again",
            path.display()
        )));
    }
    tar_padding(w, size).map_err(|e| io_err(path, e))?;
    Ok(size)
}

/// Write a backup of the open vault into `dest_dir`.
pub fn create_backup(
    session: &VaultSession,
    dest_dir: &Path,
    opts: BackupOptions<'_>,
) -> Result<BackupReport, ServiceError> {
    fs::create_dir_all(dest_dir).map_err(|e| io_err(dest_dir, e))?;
    let name = session
        .root
        .file_name()
        .map_or_else(|| "vault".to_owned(), |n| n.to_string_lossy().into_owned());
    let target = dest_dir.join(archive_name(&name, opts.now_ms));
    if target.exists() {
        return Err(ServiceError::Invalid(format!(
            "{} already exists; wait a second and try again",
            target.display()
        )));
    }
    let snapshot = dest_dir.join(format!(".studyfrog-snapshot-{}.tmp", std::process::id()));
    let _ = fs::remove_file(&snapshot);
    let result = write_archive(session, &name, (&target, &snapshot), opts);
    let _ = fs::remove_file(&snapshot);
    if result.is_err() {
        let _ = fs::remove_file(&target);
    }
    result
}

fn config_file(opts: &BackupOptions<'_>, file: &str) -> Option<PathBuf> {
    opts.config_dir
        .map(|d| d.join(file))
        .filter(|p| p.is_file())
}

fn write_archive(
    session: &VaultSession,
    vault: &str,
    (target, snapshot): (&Path, &Path),
    opts: BackupOptions<'_>,
) -> Result<BackupReport, ServiceError> {
    session
        .conn
        .execute("VACUUM INTO ?1", [snapshot.to_string_lossy().as_ref()])
        .map_err(|e| ServiceError::Io(format!("could not copy the database: {e}")))?;
    let files = vault_files(&session.root)?;
    let settings = config_file(&opts, APP_CONFIG_FILE);
    let keys = config_file(&opts, SECRETS_FILE).filter(|_| opts.include_keys);
    let mtime = u64::try_from(opts.now_ms / 1000).unwrap_or(0);
    let mut total = 0_u64;
    for f in &files {
        total += fs::metadata(session.root.join(f))
            .map_err(|e| io_err(&session.root, e))?
            .len();
    }
    let manifest = Manifest {
        format: BACKUP_FORMAT,
        created_ms: opts.now_ms,
        app_version: opts.app_version.to_owned(),
        vault_name: vault.to_owned(),
        files: u32::try_from(files.len()).unwrap_or(u32::MAX),
        bytes: total,
        has_settings: settings.is_some(),
        has_keys: keys.is_some(),
    };
    let out = File::create(target).map_err(|e| io_err(target, e))?;
    let mut w = BufWriter::new(out);
    tar_bytes(
        &mut w,
        MANIFEST_FILE,
        render_manifest(&manifest).as_bytes(),
        mtime,
    )
    .map_err(|e| io_err(target, e))?;
    add_file(&mut w, DATABASE_ENTRY, snapshot, mtime)?;
    for f in &files {
        add_file(
            &mut w,
            &format!("{VAULT_PREFIX}{f}"),
            &session.root.join(f),
            mtime,
        )?;
    }
    let providers = config_file(&opts, "providers.toml");
    for (entry, path) in [
        (SETTINGS_ENTRY, &settings),
        (KEYS_ENTRY, &keys),
        (PROVIDERS_ENTRY, &providers),
    ] {
        if let Some(p) = path {
            add_file(&mut w, entry, p, mtime)?;
        }
    }
    tar_finish(&mut w).map_err(|e| io_err(target, e))?;
    w.flush().map_err(|e| io_err(target, e))?;
    let archive_bytes = fs::metadata(target).map_err(|e| io_err(target, e))?.len();
    Ok(BackupReport {
        path: target.to_owned(),
        manifest,
        archive_bytes,
    })
}

/// Only the owner may read the keys.
pub fn set_private(path: &Path) -> Result<(), ServiceError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|e| io_err(path, e))
}
