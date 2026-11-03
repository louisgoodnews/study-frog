//! Writing `referenced_by` into the index, and reading the membership of stacks (ADR 0011).

use rusqlite::{Connection, Transaction, params};
use studyfrog_md::doc::{Doc, Kind, doc_kind};
use studyfrog_md::refs::references_of;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

/// Remember what the item's `referenced_by` says. An id that is a valid ULID is stored in its canonical form, any
/// other text as written (it will not match a stack, which the vault check reports). Stacks reference nothing.
pub(crate) fn write_refs(tx: &Transaction, item_uid: &str, doc: &Doc) -> Result<(), StoreError> {
    if doc_kind(doc) == Kind::Stack {
        return Ok(());
    }
    let mut stmt = tx
        .prepare_cached(
            "INSERT INTO item_ref (item_uid, stack_uid) VALUES (?1, ?2)
             ON CONFLICT(item_uid, stack_uid) DO UPDATE SET times = times + 1",
        )
        .map_err(sql_err)?;
    for id in references_of(doc) {
        let id = parse_ulid(&id).map_or(id, format_ulid);
        stmt.execute(params![item_uid, id]).map_err(sql_err)?;
    }
    Ok(())
}

/// The items that belong to a stack, its own and referenced, as `(item uid, is primary)`.
pub fn members_of(conn: &Connection, stack_uid: &str) -> Result<Vec<(String, bool)>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT item_uid, is_primary FROM membership WHERE stack_uid = ?1 ORDER BY item_uid",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([stack_uid], |r| Ok((r.get(0)?, r.get::<_, i64>(1)? == 1)))
        .map_err(sql_err)?;
    rows.collect::<Result<_, _>>().map_err(sql_err)
}

/// The stacks an item belongs to, as `(stack uid, is primary)`, the primary stack first.
pub fn stacks_of(conn: &Connection, item_uid: &str) -> Result<Vec<(String, bool)>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT stack_uid, is_primary FROM membership WHERE item_uid = ?1 ORDER BY is_primary DESC, stack_uid")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([item_uid], |r| Ok((r.get(0)?, r.get::<_, i64>(1)? == 1)))
        .map_err(sql_err)?;
    rows.collect::<Result<_, _>>().map_err(sql_err)
}

/// What is wrong with the references, as `(path of the item, message)`: a stack that does not exist, the item's
/// own primary stack, or a stack listed twice.
pub fn reference_problems(conn: &Connection) -> Result<Vec<(String, String)>, StoreError> {
    let sql = "
        SELECT i.path, 'references the stack ' || r.stack_uid || ', which does not exist'
          FROM item_ref r JOIN item i ON i.uid = r.item_uid
         WHERE NOT EXISTS (SELECT 1 FROM stack_identity s WHERE s.uid = r.stack_uid)
        UNION ALL
        SELECT i.path, 'lists its own stack (' || s.folder || ') in referenced_by'
          FROM item_ref r JOIN item i ON i.uid = r.item_uid
          JOIN stack_identity s ON s.uid = r.stack_uid
         WHERE i.stack_path IS s.folder
        UNION ALL
        SELECT i.path, 'lists the stack ' || r.stack_uid || ' more than once in referenced_by'
          FROM item_ref r JOIN item i ON i.uid = r.item_uid
         WHERE r.times > 1
        ORDER BY 1, 2";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(sql_err)?;
    rows.collect::<Result<_, _>>().map_err(sql_err)
}
