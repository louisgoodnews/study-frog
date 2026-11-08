//! What the analytics need to know about items besides their reviews: title, file, stack, tags and links.

use rusqlite::Connection;
use studyfrog_core::uid::Uid;

use crate::error::{StoreError, sql_err};
use crate::runs::uid_of;

/// A card or question of the index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFact {
    /// The item.
    pub uid: Uid,
    /// Its title.
    pub title: String,
    /// Its file, relative to the vault.
    pub path: String,
    /// The folder of its stack, `None` for a loose item.
    pub stack_path: Option<String>,
}

fn pairs(conn: &Connection, sql: &str) -> Result<Vec<(Uid, String)>, StoreError> {
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (uid, text) = row.map_err(sql_err)?;
        out.push((uid_of(&uid)?, text));
    }
    Ok(out)
}

/// Every card and question.
pub fn load_item_facts(conn: &Connection) -> Result<Vec<ItemFact>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT uid, title, path, stack_path FROM item WHERE kind IN ('card', 'question') ORDER BY path")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (uid, title, path, stack_path) = row.map_err(sql_err)?;
        out.push(ItemFact {
            uid: uid_of(&uid)?,
            title,
            path,
            stack_path,
        });
    }
    Ok(out)
}

/// Every (item, tag).
pub fn load_all_tags(conn: &Connection) -> Result<Vec<(Uid, String)>, StoreError> {
    pairs(conn, "SELECT uid, tag FROM tag")
}

/// Every (item, link target as written).
pub fn load_all_links(conn: &Connection) -> Result<Vec<(Uid, String)>, StoreError> {
    pairs(conn, "SELECT src_uid, target FROM link")
}
