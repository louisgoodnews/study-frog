//! What the library list needs of every item beyond what queue building reads: title, path, stack, status, dates,
//! the text, the next review and how many reviews there are.

use std::collections::HashMap;

use rusqlite::Connection;
use studyfrog_core::plan::{ItemKind, item_kind_from_name};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;

use crate::error::{StoreError, sql_err};

/// One item's metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemMeta {
    /// Note, card or question.
    pub kind: ItemKind,
    /// Title.
    pub title: String,
    /// Vault-relative path.
    pub path: String,
    /// The folder of the owning (primary) stack, `None` for the inbox.
    pub stack: Option<String>,
    /// The folders of the other stacks that reference the item.
    pub also_in: Vec<String>,
    /// The `status` word from its front matter, if there is a valid one.
    pub status: Option<String>,
    /// When it was created.
    pub created_ms: Option<i64>,
    /// When it was last updated.
    pub updated_ms: Option<i64>,
    /// The text of the item.
    pub body: String,
    /// When it is next due, if it was ever reviewed.
    pub due_ms: Option<i64>,
    /// How many reviews it has.
    pub reviews: u32,
}

/// The metadata of every item (stacks excluded), by uid.
pub fn load_item_meta(conn: &Connection) -> Result<HashMap<Uid, ItemMeta>, StoreError> {
    let sql = "SELECT i.uid, i.kind, i.title, i.path, i.stack_path, i.status, i.created_ms, i.updated_ms,
                      COALESCE(i.body, ''), COALESCE(m.due_ms, s.due_ms),
                      (SELECT COUNT(*) FROM review r WHERE r.item_uid = i.uid)
               FROM item i LEFT JOIN mem_primary m ON m.item_uid = i.uid LEFT JOIN sm2_primary s ON s.item_uid = i.uid
               WHERE i.kind != 'stack'";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                (
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                ),
                (
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<i64>>(6)?,
                    r.get::<_, Option<i64>>(7)?,
                ),
                (
                    r.get::<_, String>(8)?,
                    r.get::<_, Option<i64>>(9)?,
                    r.get::<_, u32>(10)?,
                ),
            ))
        })
        .map_err(sql_err)?;
    let mut also = crate::refs_index::referencing_folders(conn)?;
    let mut out = HashMap::new();
    for row in rows {
        let (
            id,
            kind,
            (title, path, stack),
            (status, created_ms, updated_ms),
            (body, due_ms, reviews),
        ) = row.map_err(sql_err)?;
        let corrupt = |what: &str, v: &str| StoreError::Corrupt(format!("{what}: {v:?}"));
        let uid = parse_ulid(&id).ok_or_else(|| corrupt("ulid", &id))?;
        let kind = item_kind_from_name(&kind).ok_or_else(|| corrupt("kind", &kind))?;
        out.insert(
            uid,
            ItemMeta {
                kind,
                title,
                path,
                stack,
                also_in: also.remove(&id).unwrap_or_default(),
                status,
                created_ms,
                updated_ms,
                body,
                due_ms,
                reviews,
            },
        );
    }
    Ok(out)
}

/// Whether the item belongs to the stack at `folder` or to a stack inside it: through the folder it sits in, or
/// because a stack there references it (ADR 0011).
pub fn in_stack(m: &ItemMeta, folder: &str) -> bool {
    let inside = |f: &str| f == folder || f.starts_with(&format!("{folder}/"));
    m.stack.as_deref().is_some_and(inside) || m.also_in.iter().any(|f| inside(f))
}
