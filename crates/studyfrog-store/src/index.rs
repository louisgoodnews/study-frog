//! SQL for the derived index (`file`, `item`, `link`, `tag`). All writes take a transaction.

use std::collections::HashMap;

use rusqlite::{Connection, Transaction, params};
use studyfrog_core::event::difficulty_from_name;
use studyfrog_core::priority::priority_from_name;
use studyfrog_md::doc::{Doc, doc_kind, doc_meta, doc_texts, kind_name};
use studyfrog_md::error::describe_parse_error;
use studyfrog_md::extra::extra_value;
use studyfrog_md::links::extract_wikilinks;
use studyfrog_md::ulid::format_ulid;

use crate::error::{StoreError, sql_err};
use crate::scan::ScannedFile;
use crate::stackmap::parent_folder;

/// What the index remembers about a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRow {
    /// Size in bytes at the last read.
    pub size: u64,
    /// Modification time (ns) at the last read.
    pub mtime_ns: i64,
    /// Content hash at the last read.
    pub hash: u64,
    /// Uid of the indexed item, if the file parsed.
    pub uid: Option<String>,
    /// Parse error, if it did not.
    pub error: Option<String>,
    /// Owning stack folder of the indexed item.
    pub stack: Option<String>,
}

/// Every known file by relative path.
pub fn load_rows(conn: &Connection) -> Result<HashMap<String, FileRow>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT f.path, f.size, f.mtime_ns, f.hash, f.uid, f.error, i.stack_path
             FROM file f LEFT JOIN item i ON i.path = f.path",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            let row = FileRow {
                size: r.get::<_, i64>(1)? as u64,
                mtime_ns: r.get(2)?,
                hash: r.get::<_, i64>(3)? as u64,
                uid: r.get(4)?,
                error: r.get(5)?,
                stack: r.get(6)?,
            };
            Ok((r.get::<_, String>(0)?, row))
        })
        .map_err(sql_err)?;
    rows.collect::<Result<_, _>>().map_err(sql_err)
}

/// Run a statement through the connection's prepared-statement cache.
fn run(tx: &Transaction, sql: &str, args: impl rusqlite::Params) -> Result<(), StoreError> {
    tx.prepare_cached(sql)
        .and_then(|mut s| s.execute(args))
        .map(drop)
        .map_err(sql_err)
}

/// Forget files (their items, links and tags cascade; history is untouched).
pub fn delete_files(tx: &Transaction, paths: &[String]) -> Result<(), StoreError> {
    let mut stmt = tx
        .prepare_cached("DELETE FROM file WHERE path = ?1")
        .map_err(sql_err)?;
    paths
        .iter()
        .try_for_each(|p| stmt.execute([p]).map(drop))
        .map_err(sql_err)
}

/// Refresh stat data of a file whose content is unchanged.
pub fn touch_file(tx: &Transaction, rel: &str, size: u64, mtime_ns: i64) -> Result<(), StoreError> {
    tx.prepare_cached("UPDATE file SET size = ?2, mtime_ns = ?3 WHERE path = ?1")
        .and_then(|mut s| s.execute(params![rel, size as i64, mtime_ns]))
        .map(drop)
        .map_err(sql_err)
}

/// Turn an indexed file into an error entry (for example a duplicate id).
pub fn demote_file(tx: &Transaction, rel: &str, reason: &str) -> Result<(), StoreError> {
    run(tx, "DELETE FROM item WHERE path = ?1", [rel])?;
    run(tx, "DELETE FROM stack_identity WHERE path = ?1", [rel])?;
    run(
        tx,
        "UPDATE file SET uid = NULL, error = ?2 WHERE path = ?1",
        params![rel, reason],
    )
}

/// Move an item to another stack folder.
pub fn restack_item(tx: &Transaction, rel: &str, stack: Option<&str>) -> Result<(), StoreError> {
    run(
        tx,
        "UPDATE item SET stack_path = ?2 WHERE path = ?1",
        params![rel, stack],
    )
}

const UPSERT_FILE: &str = "INSERT INTO file (path, size, mtime_ns, hash, uid, error)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
    ON CONFLICT(path) DO UPDATE SET size = ?2, mtime_ns = ?3, hash = ?4, uid = ?5, error = ?6";

/// Write the outcome of reading one file: an item with links and tags, or an error entry.
pub fn write_scanned(tx: &Transaction, f: &ScannedFile) -> Result<(), StoreError> {
    let (uid, error) = match &f.outcome {
        Ok(doc) => (Some(format_ulid(doc_meta(doc).uid)), None),
        Err(e) => (None, Some(describe_parse_error(e))),
    };
    let file = &f.file;
    tx.prepare_cached(UPSERT_FILE)
        .and_then(|mut s| {
            s.execute(params![
                file.rel,
                file.size as i64,
                file.mtime_ns,
                f.hash as i64,
                uid,
                error
            ])
        })
        .map_err(sql_err)?;
    run(tx, "DELETE FROM item WHERE path = ?1", [&file.rel])?;
    run(
        tx,
        "DELETE FROM stack_identity WHERE path = ?1",
        [&file.rel],
    )?;
    match &f.outcome {
        Ok(doc) => write_item(tx, &file.rel, f.stack.as_deref(), doc),
        Err(_) => Ok(()),
    }
}

/// The value of a front-matter word key when it is one of the known words, else `NULL`.
fn word_of(doc: &Doc, key: &str, known: fn(&str) -> bool) -> Option<String> {
    extra_value(doc, key).filter(|w| known(w))
}

fn write_item(
    tx: &Transaction,
    rel: &str,
    stack: Option<&str>,
    doc: &Doc,
) -> Result<(), StoreError> {
    let meta = doc_meta(doc);
    let uid = format_ulid(meta.uid);
    run(tx, "DELETE FROM item WHERE uid = ?1", [&uid])?;
    run(
        tx,
        "INSERT INTO item (uid, path, kind, title, stack_path, created_ms, updated_ms, priority, difficulty, subjects)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            uid,
            rel,
            kind_name(doc_kind(doc)),
            meta.title,
            stack,
            meta.created.map(|t| t.0),
            meta.updated.map(|t| t.0),
            word_of(doc, "priority", |w| priority_from_name(w).is_some()),
            word_of(doc, "difficulty", |w| difficulty_from_name(w).is_some()),
            extra_value(doc, "subjects")
        ],
    )?;
    write_tags(tx, &uid, &meta.tags)?;
    if matches!(doc, Doc::Stack(_)) {
        run(
            tx,
            "INSERT INTO stack_identity (path, uid, folder) VALUES (?1, ?2, ?3)",
            params![rel, uid, parent_folder(rel)],
        )?;
    }
    write_links(tx, &uid, doc)
}

fn write_tags(tx: &Transaction, uid: &str, tags: &[String]) -> Result<(), StoreError> {
    let mut stmt = tx
        .prepare_cached("INSERT OR IGNORE INTO tag (uid, tag) VALUES (?1, ?2)")
        .map_err(sql_err)?;
    tags.iter()
        .try_for_each(|t| stmt.execute(params![uid, t]).map(drop))
        .map_err(sql_err)
}

fn write_links(tx: &Transaction, uid: &str, doc: &Doc) -> Result<(), StoreError> {
    let mut stmt = tx
        .prepare_cached(
            "INSERT INTO link (src_uid, seq, target, heading, label, line) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .map_err(sql_err)?;
    let links = doc_texts(doc).into_iter().flat_map(extract_wikilinks);
    for (seq, l) in links.enumerate() {
        stmt.execute(params![
            uid,
            seq as i64,
            l.target,
            l.heading,
            l.label,
            l.line as i64
        ])
        .map_err(sql_err)?;
    }
    Ok(())
}

/// Position of every indexed item in path order (the library order used by sequential runs).
pub fn item_positions(
    conn: &Connection,
) -> Result<HashMap<studyfrog_core::uid::Uid, u32>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT uid FROM item ORDER BY path")
        .map_err(sql_err)?;
    let ids = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(sql_err)?;
    let mut out = HashMap::new();
    for (pos, id) in ids.enumerate() {
        let id = id.map_err(sql_err)?;
        let uid = studyfrog_md::ulid::parse_ulid(&id)
            .ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?;
        out.insert(uid, pos as u32);
    }
    Ok(out)
}
