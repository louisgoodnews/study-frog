//! Item keys in the vault (STORY-0096): a new item gets the next key of its kind when it is made,
//! and items made before keys existed get theirs only through an explicit, confirmed action that can
//! be undone like a bulk edit.

use std::fs;

use rusqlite::Connection;
use studyfrog_core::item_key::KEY_FIELD;
use studyfrog_core::time::UnixMs;
use studyfrog_md::doc::{Doc, doc_kind, kind_name};
use studyfrog_md::extra::{extra_value, with_extra};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;
use studyfrog_store::item_keys::{KeyHit, allocate_key, items_without_key};

use crate::bulk::{BulkDone, Snapshot};
use crate::create::{refresh, write_new};
use crate::session::{ServiceError, VaultSession};

/// The document with a key of its own: kept if it has one, else the next key of its kind (a diary
/// entry gets none).
pub fn with_new_key(conn: &Connection, doc: Doc) -> Result<Doc, ServiceError> {
    if extra_value(&doc, KEY_FIELD).is_some() {
        return Ok(doc);
    }
    let key = allocate_key(conn, kind_name(doc_kind(&doc))).map_err(ServiceError::Store)?;
    Ok(match key {
        Some(k) => with_extra(&doc, KEY_FIELD, Some(&k)),
        None => doc,
    })
}

/// The stacks and items that have no key yet, as the confirmation lists them.
pub fn keyless_items(session: &VaultSession) -> Result<Vec<KeyHit>, ServiceError> {
    items_without_key(&session.conn).map_err(ServiceError::Store)
}

fn stamped(doc: Doc, now: UnixMs) -> Doc {
    let mut doc = doc;
    match &mut doc {
        Doc::Stack(d) => d.meta.updated = Some(now),
        Doc::Note(d) => d.meta.updated = Some(now),
        Doc::Card(d) => d.meta.updated = Some(now),
        Doc::Question(d) => d.meta.updated = Some(now),
        Doc::Diary(d) => d.meta.updated = Some(now),
    }
    doc
}

/// Give one keyless item its key; its text before goes to `undo`.
fn assign_one(
    session: &VaultSession,
    hit: &KeyHit,
    now: UnixMs,
    undo: &mut Snapshot,
) -> Result<(), ServiceError> {
    let io = |e: std::io::Error| ServiceError::Io(format!("{}: {e}", hit.path));
    let file = session.root.join(&hit.path);
    let before = fs::read_to_string(&file).map_err(io)?;
    let doc =
        parse_doc(&before).map_err(|e| ServiceError::Invalid(format!("{}: {e}", hit.path)))?;
    if extra_value(&doc, KEY_FIELD).is_some() {
        return Ok(()); // given one since the index was read
    }
    let doc = stamped(with_new_key(&session.conn, doc)?, now);
    let text = render_doc(&doc).map_err(|e| ServiceError::Invalid(e.to_string()))?;
    undo.0.push((hit.path.clone(), Some(before)));
    write_new(&file, &text)
}

/// Give every stack and item without a key the next key of its kind, oldest first, and reindex once.
/// Files that cannot be changed are listed and left alone; the snapshot puts every change back.
pub fn assign_missing_keys(
    session: &mut VaultSession,
    now: UnixMs,
) -> Result<BulkDone, ServiceError> {
    let missing = keyless_items(session)?;
    let (mut undo, mut skipped, mut changed) = (Snapshot::default(), Vec::new(), 0);
    for hit in &missing {
        match assign_one(session, hit, now, &mut undo) {
            Ok(()) => changed += 1,
            Err(e) => skipped.push(e.to_string()),
        }
    }
    let info = refresh(session)?;
    Ok(BulkDone {
        changed,
        skipped,
        undo,
        info,
    })
}
