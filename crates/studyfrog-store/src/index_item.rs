//! Writing one parsed item into the index: its row (with key and colour), tags, references and links.

use rusqlite::{Transaction, params};
use studyfrog_core::event::difficulty_from_name;
use studyfrog_core::item_color::{COLOR_FIELD, normalize_color};
use studyfrog_core::item_key::{KEY_FIELD, normalize_key};
use studyfrog_core::priority::priority_from_name;
use studyfrog_core::status::status_from_name;
use studyfrog_md::doc::{Doc, doc_kind, doc_meta, doc_texts, kind_name};
use studyfrog_md::extra::{extra_raw, extra_value};
use studyfrog_md::links::extract_wikilinks;
use studyfrog_md::ulid::format_ulid;

use crate::error::{StoreError, sql_err};
use crate::index::run;
use crate::stackmap::parent_folder;

/// Whether the index has the key and colour columns (schema step 24); a migration test may write
/// files into an older schema.
fn has_keys(tx: &Transaction) -> bool {
    tx.pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
        .is_ok_and(|v| v >= 24)
}

/// The value of a front-matter word key when it is one of the known words, else `NULL`.
fn word_of(doc: &Doc, key: &str, known: fn(&str) -> bool) -> Option<String> {
    extra_value(doc, key).filter(|w| known(w))
}

pub(crate) fn write_item(
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
        "INSERT INTO item (uid, path, kind, title, stack_path, created_ms, updated_ms, priority, difficulty, subjects, source, status, body)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
            extra_raw(doc, "subjects"),
            extra_value(doc, "source"),
            word_of(doc, "status", |w| status_from_name(w).is_some()),
            doc_texts(doc).join("\n").trim().to_owned()
        ],
    )?;
    if has_keys(tx) {
        run(
            tx,
            "UPDATE item SET key = ?2, color = ?3 WHERE uid = ?1",
            params![
                uid,
                extra_value(doc, KEY_FIELD).and_then(|k| normalize_key(&k)),
                extra_value(doc, COLOR_FIELD).and_then(|c| normalize_color(&c))
            ],
        )?;
    }
    write_tags(tx, &uid, &meta.tags)?;
    crate::refs_index::write_refs(tx, &uid, doc)?;
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
