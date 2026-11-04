//! Item lookups over the index: listing, queue-building rows, paths and title resolution.

use rusqlite::Connection;
use studyfrog_core::event::Difficulty;
use studyfrog_core::priority::Priority;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;

use crate::error::{StoreError, sql_err};
use crate::item_rows::{ITEM_COLS, item_row, raw_item};
use crate::traits::{inherited, own_of, stack_traits};

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
    /// Its status: the stored one, else derived from whether it was ever reviewed.
    pub status: studyfrog_core::status::Status,
}

/// Items directly in `folder` (`None` for the inbox), without the stack file, ordered by path.
pub fn list_items(conn: &Connection, folder: Option<&str>) -> Result<Vec<ItemRow>, StoreError> {
    let sql = format!(
        "SELECT {ITEM_COLS}
               FROM item i LEFT JOIN mem_primary m ON m.item_uid = i.uid LEFT JOIN sm2_primary s ON s.item_uid = i.uid
               WHERE i.stack_path IS ?1 AND i.kind != 'stack' ORDER BY i.path"
    );
    let mut stmt = conn.prepare(&sql).map_err(sql_err)?;
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
    let sql = format!(
        "SELECT {ITEM_COLS}
               FROM item i LEFT JOIN mem_primary m ON m.item_uid = i.uid LEFT JOIN sm2_primary s ON s.item_uid = i.uid
               WHERE i.kind != 'stack' AND i.title LIKE ?1 ESCAPE '\\'
               ORDER BY (i.title LIKE ?2 ESCAPE '\\') DESC, i.title COLLATE NOCASE, i.path LIMIT ?3"
    );
    let mut stmt = conn.prepare(&sql).map_err(sql_err)?;
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
    /// The other stacks that reference it.
    pub also_in: Vec<Uid>,
    /// Tags.
    pub tags: Vec<String>,
    /// Position in path order, the base order of sequential runs.
    pub position: u32,
    /// How much it matters: its own, else its stack's, else medium.
    pub priority: Priority,
    /// How hard its owner says it is: its own, else its stack's, if either says.
    pub difficulty: Option<Difficulty>,
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
    let mut also = crate::refs_index::referencing_stacks(conn)?;
    let stacks = stack_traits(conn)?;
    let sql = "SELECT i.uid, i.kind, s.uid, i.priority, i.difficulty, i.stack_path
               FROM item i LEFT JOIN stack_setting s ON s.folder = i.stack_path
               WHERE i.kind != 'stack' ORDER BY i.path";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                (
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                ),
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for (position, row) in rows.enumerate() {
        let (id, kind, stack, (p, d, folder)) = row.map_err(sql_err)?;
        let (priority, difficulty) = inherited(own_of(p, d), folder.as_deref(), &stacks);
        let corrupt = |what: &str, v: &str| StoreError::Corrupt(format!("{what}: {v:?}"));
        out.push(IndexedItem {
            uid: parse_ulid(&id).ok_or_else(|| corrupt("ulid", &id))?,
            kind: studyfrog_core::plan::item_kind_from_name(&kind)
                .ok_or_else(|| corrupt("kind", &kind))?,
            stack: stack.as_deref().map_or(Ok(Uid(0)), |s| {
                parse_ulid(s).ok_or_else(|| corrupt("ulid", s))
            })?,
            also_in: also.remove(&id).unwrap_or_default(),
            tags: tags.remove(&id).unwrap_or_default(),
            position: position as u32,
            priority,
            difficulty,
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

/// The items (not stacks) whose text links to `title` with a wikilink, without `except` itself.
pub fn linking_to(conn: &Connection, title: &str, except: Uid) -> Result<Vec<ItemRow>, StoreError> {
    let sql = format!(
        "SELECT DISTINCT {ITEM_COLS}
               FROM link l JOIN item i ON i.uid = l.src_uid
               LEFT JOIN mem_primary m ON m.item_uid = i.uid LEFT JOIN sm2_primary s ON s.item_uid = i.uid
               WHERE l.target = ?1 COLLATE NOCASE AND i.kind != 'stack' AND i.uid != ?2
               ORDER BY i.title COLLATE NOCASE, i.path"
    );
    let mut stmt = conn.prepare(&sql).map_err(sql_err)?;
    let params = (title.trim(), studyfrog_md::ulid::format_ulid(except));
    let rows = stmt.query_map(params, raw_item).map_err(sql_err)?;
    rows.map(|r| item_row(r.map_err(sql_err)?)).collect()
}

/// The items due at `now_ms` or earlier, the most overdue first, at most `limit`.
pub fn due_items(conn: &Connection, now_ms: i64, limit: usize) -> Result<Vec<ItemRow>, StoreError> {
    let sql = format!(
        "SELECT {ITEM_COLS}
               FROM item i LEFT JOIN mem_primary m ON m.item_uid = i.uid LEFT JOIN sm2_primary s ON s.item_uid = i.uid
               WHERE i.kind != 'stack' AND COALESCE(m.due_ms, s.due_ms) <= ?1 ORDER BY due, i.path LIMIT ?2"
    );
    let mut stmt = conn.prepare(&sql).map_err(sql_err)?;
    let rows = stmt
        .query_map((now_ms, limit as i64), raw_item)
        .map_err(sql_err)?;
    rows.map(|r| item_row(r.map_err(sql_err)?)).collect()
}

/// The items edited most recently (by the time in their front matter, else when they were created),
/// newest first.
pub fn recent_items(conn: &Connection, limit: usize) -> Result<Vec<ItemRow>, StoreError> {
    let sql = format!(
        "SELECT {ITEM_COLS}
               FROM item i LEFT JOIN mem_primary m ON m.item_uid = i.uid LEFT JOIN sm2_primary s ON s.item_uid = i.uid
               WHERE i.kind != 'stack' ORDER BY COALESCE(i.updated_ms, i.created_ms, 0) DESC, i.path LIMIT ?1"
    );
    let mut stmt = conn.prepare(&sql).map_err(sql_err)?;
    let rows = stmt.query_map([limit as i64], raw_item).map_err(sql_err)?;
    rows.map(|r| item_row(r.map_err(sql_err)?)).collect()
}

/// How many items of the stack in `folder` and of every stack below it are in each status, in menu order.
/// The status is the stored one, else `learning` for an item with a review and `new` for one without
/// (the same rule as [`studyfrog_core::status::effective_status`]).
pub fn status_counts(
    conn: &Connection,
    folder: &str,
) -> Result<Vec<(studyfrog_core::status::Status, u32)>, StoreError> {
    let escaped = folder
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let sql = "SELECT COALESCE(i.status, CASE WHEN EXISTS(SELECT 1 FROM review r WHERE r.item_uid = i.uid)
                    THEN 'learning' ELSE 'new' END), COUNT(*)
               FROM item i
               WHERE i.kind != 'stack' AND (i.stack_path = ?1 OR i.stack_path LIKE ?2 ESCAPE '\\')
               GROUP BY 1";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map((folder, format!("{escaped}/%")), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?))
        })
        .map_err(sql_err)?;
    let found: Vec<(String, u32)> = rows.collect::<Result<_, _>>().map_err(sql_err)?;
    let count_of = |word: &str| found.iter().find(|(w, _)| w == word).map_or(0, |(_, n)| *n);
    Ok(studyfrog_core::status::STATUSES
        .into_iter()
        .map(|s| (s, count_of(studyfrog_core::status::status_name(s))))
        .collect())
}
