//! Incremental reconciliation of the vault with the index.
//!
//! Cost model: unchanged files (same size and mtime) are only stat-ed, touched-but-identical files
//! are read and hashed but not parsed, everything else is parsed. One transaction applies it all.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use rusqlite::Connection;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::doc_meta;
use studyfrog_md::ulid::{format_ulid, parse_ulid};
use studyfrog_md::validate::find_duplicate_ids;

use crate::error::{StoreError, sql_err};
use crate::index::{
    FileRow, delete_files, demote_file, load_rows, restack_item, touch_file, write_scanned,
};
use crate::scan::{ScannedFile, duplicate_error, parse_bytes, read_error, read_file};
use crate::stackmap::{owning_stack, stack_folders};
use crate::vault::{VaultFile, list_markdown};

/// What a reindex did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReindexReport {
    /// Markdown files in the vault.
    pub files: usize,
    /// Skipped because size and mtime matched.
    pub unchanged: usize,
    /// Read but identical in content; only stat data refreshed.
    pub touched: usize,
    /// Parsed successfully and (re)indexed.
    pub indexed: usize,
    /// Recorded as errors (parse failures and duplicate ids).
    pub failed: usize,
    /// Dropped from the index because the file is gone.
    pub removed: usize,
    /// Items moved to another stack because a `_stack.md` appeared or vanished.
    pub restacked: usize,
}

enum Processed {
    Touched(VaultFile),
    Parsed(Box<ScannedFile>),
}

/// A file needs reading when it is new, had an error, or its size or mtime moved.
fn needs_read(file: &VaultFile, row: Option<&FileRow>) -> bool {
    row.is_none_or(|r| r.error.is_some() || r.size != file.size || r.mtime_ns != file.mtime_ns)
}

fn process(file: &VaultFile, row: Option<&FileRow>, folders: &HashSet<String>) -> Processed {
    let (hash, outcome) = match read_file(file) {
        Ok((_, hash)) if row.is_some_and(|r| r.error.is_none() && r.hash == hash) => {
            return Processed::Touched(file.clone());
        }
        Ok((bytes, hash)) => (hash, parse_bytes(&bytes)),
        Err(reason) => (0, Err(read_error(reason))),
    };
    let stack = owning_stack(&file.rel, folders);
    Processed::Parsed(Box::new(ScannedFile {
        file: file.clone(),
        hash,
        stack,
        outcome,
    }))
}

/// Every (path, uid) that would be indexed, to find ids shared between files.
fn live_ids(
    listed: &[VaultFile],
    rows: &HashMap<String, FileRow>,
    processed: &[Processed],
) -> Vec<(String, Uid)> {
    let reread: HashSet<&str> = processed.iter().map(path_of).collect();
    let kept = listed
        .iter()
        .filter(|f| !reread.contains(f.rel.as_str()))
        .filter_map(|f| {
            let uid = rows.get(&f.rel)?.uid.as_deref()?;
            Some((f.rel.clone(), parse_ulid(uid)?))
        });
    let touched = processed.iter().filter_map(|p| match p {
        Processed::Touched(f) => Some((
            f.rel.clone(),
            parse_ulid(rows.get(&f.rel)?.uid.as_deref()?)?,
        )),
        Processed::Parsed(s) => s
            .outcome
            .as_ref()
            .ok()
            .map(|d| (s.file.rel.clone(), doc_meta(d).uid)),
    });
    kept.chain(touched).collect()
}

fn path_of(p: &Processed) -> &str {
    match p {
        Processed::Touched(f) => &f.rel,
        Processed::Parsed(s) => &s.file.rel,
    }
}

/// Reason per path for every file whose uid is used more than once.
fn clashes(ids: &[(String, Uid)]) -> HashMap<String, String> {
    let mut reasons = HashMap::new();
    for (uid, paths) in find_duplicate_ids(ids) {
        let names: Vec<&str> = paths.iter().map(|p| p.as_str()).collect();
        let reason = format!(
            "id {} is used by several files: {}",
            format_ulid(uid),
            names.join(", ")
        );
        names.iter().for_each(|n| {
            reasons.insert((*n).to_owned(), reason.clone());
        });
    }
    reasons
}

/// Bring the index in line with the vault.
pub fn reindex(conn: &mut Connection, root: &Path) -> Result<ReindexReport, StoreError> {
    reindex_files(conn, &list_markdown(root)?)
}

/// Like [`reindex`] for a vault listing the caller already has (see `poll`).
pub fn reindex_files(
    conn: &mut Connection,
    listed: &[VaultFile],
) -> Result<ReindexReport, StoreError> {
    let rows = load_rows(conn)?;
    let folders = stack_folders(listed);
    let processed: Vec<Processed> = listed
        .iter()
        .filter(|f| needs_read(f, rows.get(&f.rel)))
        .map(|f| process(f, rows.get(&f.rel), &folders))
        .collect();
    let clash = clashes(&live_ids(listed, &rows, &processed));
    apply(conn, listed, &rows, processed, &clash, &folders)
}

fn apply(
    conn: &mut Connection,
    listed: &[VaultFile],
    rows: &HashMap<String, FileRow>,
    processed: Vec<Processed>,
    clash: &HashMap<String, String>,
    folders: &HashSet<String>,
) -> Result<ReindexReport, StoreError> {
    let present: HashSet<&str> = listed.iter().map(|f| f.rel.as_str()).collect();
    let gone: Vec<String> = rows
        .keys()
        .filter(|p| !present.contains(p.as_str()))
        .cloned()
        .collect();
    let reread: HashSet<String> = processed.iter().map(|p| path_of(p).to_owned()).collect();
    let mut report = ReindexReport {
        files: listed.len(),
        removed: gone.len(),
        ..Default::default()
    };
    report.unchanged = listed.len() - reread.len();
    let tx = conn.transaction().map_err(sql_err)?;
    delete_files(&tx, &gone)?;
    for p in processed {
        match p {
            Processed::Touched(f) if !clash.contains_key(&f.rel) => {
                touch_file(&tx, &f.rel, f.size, f.mtime_ns)?;
                report.touched += 1;
            }
            Processed::Touched(f) => demote_file(&tx, &f.rel, &clash[&f.rel])?,
            Processed::Parsed(mut s) => {
                if let Some(reason) = clash.get(&s.file.rel) {
                    s.outcome = Err(duplicate_error(reason.clone()));
                }
                write_scanned(&tx, &s)?;
                if s.outcome.is_ok() {
                    report.indexed += 1
                } else {
                    report.failed += 1
                }
            }
        }
    }
    report.failed += demote_unread(&tx, listed, rows, &reread, clash)?;
    report.restacked = restack(&tx, listed, rows, &reread, folders)?;
    tx.commit().map_err(sql_err)?;
    Ok(report)
}

/// Previously indexed files that were not re-read but now clash with another file.
fn demote_unread(
    tx: &rusqlite::Transaction,
    listed: &[VaultFile],
    rows: &HashMap<String, FileRow>,
    reread: &HashSet<String>,
    clash: &HashMap<String, String>,
) -> Result<usize, StoreError> {
    let mut count = 0;
    for f in listed.iter().filter(|f| !reread.contains(&f.rel)) {
        let indexed = rows.get(&f.rel).is_some_and(|r| r.uid.is_some());
        if let (true, Some(reason)) = (indexed, clash.get(&f.rel)) {
            demote_file(tx, &f.rel, reason)?;
            count += 1;
        }
    }
    Ok(count)
}

/// Unread, still-indexed files whose owning stack folder changed.
fn restack(
    tx: &rusqlite::Transaction,
    listed: &[VaultFile],
    rows: &HashMap<String, FileRow>,
    reread: &HashSet<String>,
    folders: &HashSet<String>,
) -> Result<usize, StoreError> {
    let mut count = 0;
    for f in listed.iter().filter(|f| !reread.contains(&f.rel)) {
        let Some(row) = rows.get(&f.rel).filter(|r| r.uid.is_some()) else {
            continue;
        };
        let now = owning_stack(&f.rel, folders);
        if now != row.stack {
            restack_item(tx, &f.rel, now.as_deref())?;
            count += 1;
        }
    }
    Ok(count)
}
