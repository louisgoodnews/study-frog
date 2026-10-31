//! Performing the backup and restore effects: the dialogs, then the service.

use std::path::PathBuf;

use studyfrog_service::backup::{BackupOptions, create_backup};
use studyfrog_service::restore::{read_manifest, restore_backup};

use crate::backup_ui::{BackupMsg, Chosen};
use crate::exec::Worker;
use crate::msg::Msg;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn folder(w: &Worker) -> Option<PathBuf> {
    w.folder_picker.and_then(|pick| pick())
}

/// Ask where to put the backup and write it.
pub fn backup(w: &Worker, include_keys: bool, now_ns: i64) -> Msg {
    let (Some(session), Some(dest)) = (&w.session, folder(w)) else {
        return Msg::Backup(BackupMsg::Done(
            w.session
                .as_ref()
                .map_or_else(|| Err("no vault is open".to_owned()), |_| Ok(None)),
        ));
    };
    let opts = BackupOptions {
        config_dir: w.config_dir.as_deref(),
        include_keys,
        now_ms: now_ns / 1_000_000,
        app_version: VERSION,
    };
    let result = create_backup(session, &dest, opts)
        .map(|r| {
            Some(format!(
                "Saved {} ({} files, {} KB)",
                r.path.display(),
                r.manifest.files,
                r.archive_bytes / 1024
            ))
        })
        .map_err(|e| e.to_string());
    Msg::Backup(BackupMsg::Done(result))
}

/// Ask for a backup file and read what it holds.
pub fn choose(w: &Worker) -> Msg {
    let result = match w.file_picker.and_then(|pick| pick(&["tar"])) {
        None => Ok(None),
        Some(path) => read_manifest(&path)
            .map(|manifest| {
                Some(Chosen {
                    path: path.display().to_string(),
                    manifest,
                })
            })
            .map_err(|e| e.to_string()),
    };
    Msg::Backup(BackupMsg::Chosen(result))
}

/// Ask for an empty folder and unpack the backup there.
pub fn restore(w: &Worker, archive: &str, with_settings: bool) -> Msg {
    let result = match folder(w) {
        None => Ok(None),
        Some(target) => {
            let config = w.config_dir.as_deref().filter(|_| with_settings);
            restore_backup(&PathBuf::from(archive), &target, config)
                .map(|r| Some(r.vault.display().to_string()))
                .map_err(|e| e.to_string())
        }
    };
    Msg::Backup(BackupMsg::Restored(result))
}
