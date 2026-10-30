//! Item lookups over the index: listing, queue-building rows, paths and title resolution.

use rusqlite::Connection;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;

use crate::error::{StoreError, sql_err};

/// One item of a stack (or of the inbox) as the UI lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRow {
    /// Item identity.
    pub uid: Uid,
    /// Title.
    pub title: String,
    /// Note, card or question.
    pub kind: studyfrog_core::plan::ItemKind,
    /// Vault-relative path of the file.
    pub path: String,
    /// When the item is next due; `None` for items that were never reviewed.
    pub due: Option<studyfrog_core::time::UnixMs>,
}

type RawItem = (String, String, String, String, Option<i64>);

fn raw_item(r: &rusqlite::Row<'_>) -> rusqlite::Result<RawItem> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
}

fn item_row((id, title, kind, path, due): RawItem) -> Result<ItemRow, StoreError> {
    let uid = parse_ulid(&id).ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?;
    let kind = studyfrog_core::plan::item_kind_from_name(&kind)
        .ok_or_else(|| StoreError::Corrupt(format!("kind: {kind:?}")))?;
    Ok(ItemRow {
        uid,
        title,
        kind,
        path,
        due: due.map(studyfrog_core::time::UnixMs),
    })
}

/// Items directly in `folder` (`None` for the inbox), without the stack file, ordered by path.
pub fn list_items(conn: &Connection, folder: Option<&str>) -> Result<Vec<ItemRow>, StoreError> {
    let sql = "SELECT i.uid, i.title, i.kind, i.path, COALESCE(m.due_ms, s.due_ms)
               FROM item i LEFT JOIN mem_state m ON m.item_uid = i.uid LEFT JOIN sm2_state s ON s.item_uid = i.uid
               WHERE i.stack_path IS ?1 AND i.kind != 'stack' ORDER BY i.path";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt.query_map([folder], raw_item).map_err(sql_err)?;
    rows.map(|r| item_row(r.map_err(sql_err)?)).collect()
}

/// Items anywhere in the vault whose title contains `query` (case-insensitive; `%` and `_` are taken
/// literally), titles that start with it first, at most `limit`. An empty query finds nothing.
pub fn search_items(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<ItemRow>, StoreError> {
    let needle = query.trim();
    if needle.is_empty() {
        return Ok(Vec::new());
    }
    let escaped = needle
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let sql = "SELECT i.uid, i.title, i.kind, i.path, COALESCE(m.due_ms, s.due_ms)
               FROM item i LEFT JOIN mem_state m ON m.item_uid = i.uid LEFT JOIN sm2_state s ON s.item_uid = i.uid
               WHERE i.kind != 'stack' AND i.title LIKE ?1 ESCAPE '\\'
               ORDER BY (i.title LIKE ?2 ESCAPE '\\') DESC, i.title COLLATE NOCASE, i.path LIMIT ?3";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let params = (format!("%{escaped}%"), format!("{escaped}%"), limit as i64);
    let rows = stmt.query_map(params, raw_item).map_err(sql_err)?;
    rows.map(|r| item_row(r.map_err(sql_err)?)).collect()
}

/// What queue building needs to know about an indexed item (no file content).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedItem {
    /// Item identity.
    pub uid: Uid,
    /// Note, card or question.
    pub kind: studyfrog_core::plan::ItemKind,
    /// Owning stack, `Uid(0)` for the inbox.
    pub stack: Uid,
    /// Tags.
    pub tags: Vec<String>,
    /// Position in path order, the base order of sequential runs.
    pub position: u32,
}

fn tags_by_item(
    conn: &Connection,
) -> Result<std::collections::HashMap<String, Vec<String>>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT uid, tag FROM tag ORDER BY uid, tag")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(sql_err)?;
    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for row in rows {
        let (uid, tag) = row.map_err(sql_err)?;
        map.entry(uid).or_default().push(tag);
    }
    Ok(map)
}

/// Every studiable item (stacks excluded) in path order, with its stack and tags.
pub fn load_indexed_items(conn: &Connection) -> Result<Vec<IndexedItem>, StoreError> {
    let mut tags = tags_by_item(conn)?;
    let sql = "SELECT i.uid, i.kind, s.uid FROM item i LEFT JOIN stack_setting s ON s.folder = i.stack_path
               WHERE i.kind != 'stack' ORDER BY i.path";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for (position, row) in rows.enumerate() {
        let (id, kind, stack) = row.map_err(sql_err)?;
        let corrupt = |what: &str, v: &str| StoreError::Corrupt(format!("{what}: {v:?}"));
        out.push(IndexedItem {
            uid: parse_ulid(&id).ok_or_else(|| corrupt("ulid", &id))?,
            kind: studyfrog_core::plan::item_kind_from_name(&kind)
                .ok_or_else(|| corrupt("kind", &kind))?,
            stack: stack.as_deref().map_or(Ok(Uid(0)), |s| {
                parse_ulid(s).ok_or_else(|| corrupt("ulid", s))
            })?,
            tags: tags.remove(&id).unwrap_or_default(),
            position: position as u32,
        });
    }
    Ok(out)
}

/// Vault-relative path of an indexed item, if it is (still) in the index.
pub fn item_path(conn: &Connection, uid: Uid) -> Result<Option<String>, StoreError> {
    let sql = "SELECT path FROM item WHERE uid = ?1";
    match conn.query_row(sql, [studyfrog_md::ulid::format_ulid(uid)], |r| r.get(0)) {
        Ok(path) => Ok(Some(path)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(sql_err(e)),
    }
}

/// The first non-stack item whose title equals `target`, ignoring case (path order breaks ties).
/// Aliases are not indexed yet, so only titles resolve.
pub fn resolve_title(conn: &Connection, target: &str) -> Result<Option<Uid>, StoreError> {
    let sql = "SELECT uid FROM item WHERE title = ?1 COLLATE NOCASE AND kind != 'stack' ORDER BY path LIMIT 1";
    match conn.query_row(sql, [target.trim()], |r| r.get::<_, String>(0)) {
        Ok(id) => parse_ulid(&id)
            .map(Some)
            .ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}"))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(sql_err(e)),
    }
}
