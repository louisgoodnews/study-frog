//! `_stack.toml` files: listing, reading and keeping the `stack_config` table in step.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use rusqlite::{Connection, params};
use studyfrog_config::stack::{STACK_CONFIG_FILE, parse_stack_settings};
use studyfrog_core::scheduler::scheduler_name;
use studyfrog_core::settings::StackSettings;

use crate::error::{StoreError, sql_err};
use crate::stackmap::parent_folder;
use crate::vault::VaultFile;

fn stat(root: &Path, rel: String) -> Option<VaultFile> {
    let abs = root.join(&rel);
    let meta = fs::metadata(&abs).ok().filter(|m| m.is_file())?;
    let mtime_ns = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX));
    Some(VaultFile {
        rel,
        abs,
        size: meta.len(),
        mtime_ns,
    })
}

/// The `_stack.toml` of every stack folder that has one. One `stat` per stack, no directory walk.
pub fn list_stack_configs(root: &Path, folders: &HashSet<String>) -> Vec<VaultFile> {
    let mut found: Vec<VaultFile> = folders
        .iter()
        .filter_map(|f| {
            let rel = if f.is_empty() {
                STACK_CONFIG_FILE.to_owned()
            } else {
                format!("{f}/{STACK_CONFIG_FILE}")
            };
            stat(root, rel)
        })
        .collect();
    found.sort_by(|a, b| a.rel.cmp(&b.rel));
    found
}

/// Read and validate one settings file; the error message includes the line when known.
pub fn read_stack_settings(file: &VaultFile) -> Result<StackSettings, String> {
    let text = fs::read_to_string(&file.abs).map_err(|e| format!("cannot read file: {e}"))?;
    parse_stack_settings(&text).map_err(|e| e.to_string())
}

type Stored = HashMap<String, (i64, i64)>;

fn stored_stats(conn: &Connection) -> Result<Stored, StoreError> {
    let mut stmt = conn
        .prepare("SELECT folder, size, mtime_ns FROM stack_config")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?))))
        .map_err(sql_err)?;
    rows.collect::<Result<_, _>>().map_err(sql_err)
}

fn write_config(conn: &Connection, file: &VaultFile) -> Result<bool, StoreError> {
    let (s, error) = match read_stack_settings(file) {
        Ok(s) => (s, None),
        Err(message) => (StackSettings::default(), Some(message)),
    };
    conn.execute(
        "INSERT INTO stack_config (folder, size, mtime_ns, scheduler, request_retention, daily_new_limit, daily_review_limit, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(folder) DO UPDATE SET size = ?2, mtime_ns = ?3, scheduler = ?4, request_retention = ?5,
             daily_new_limit = ?6, daily_review_limit = ?7, error = ?8",
        params![
            parent_folder(&file.rel),
            file.size as i64,
            file.mtime_ns,
            s.scheduler.map(scheduler_name),
            s.request_retention,
            s.daily_new_limit,
            s.daily_review_limit,
            error
        ],
    )
    .map_err(sql_err)?;
    Ok(error.is_some())
}

/// What a sync changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConfigSync {
    /// Settings files read again (new or changed).
    pub updated: usize,
    /// Of those, how many were invalid (their stack uses the defaults).
    pub invalid: usize,
    /// Rows removed because the file or the stack is gone.
    pub removed: usize,
}

/// Bring `stack_config` in line with the listed `_stack.toml` files.
pub fn sync_stack_configs(
    conn: &Connection,
    files: &[VaultFile],
) -> Result<ConfigSync, StoreError> {
    let stored = stored_stats(conn)?;
    let mut sync = ConfigSync::default();
    for file in files {
        let folder = parent_folder(&file.rel);
        if stored.get(folder) == Some(&(file.size as i64, file.mtime_ns)) {
            continue;
        }
        sync.updated += 1;
        sync.invalid += usize::from(write_config(conn, file)?);
    }
    let present: HashSet<&str> = files.iter().map(|f| parent_folder(&f.rel)).collect();
    for folder in stored.keys().filter(|f| !present.contains(f.as_str())) {
        conn.execute("DELETE FROM stack_config WHERE folder = ?1", [folder])
            .map_err(sql_err)?;
        sync.removed += 1;
    }
    Ok(sync)
}
