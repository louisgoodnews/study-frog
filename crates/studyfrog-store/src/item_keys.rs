//! Item keys over the index: handing out the next key of a kind, finding an item by its key, and
//! listing the items that have none yet.

use rusqlite::{Connection, OptionalExtension, params};
use studyfrog_core::item_key::{key_prefix, next_key, normalize_key, parse_key};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;

use crate::error::{StoreError, sql_err};

/// An item found by its key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHit {
    /// The item.
    pub uid: Uid,
    /// `stack`, `note`, `card` or `question`.
    pub kind: String,
    /// Its title.
    pub title: String,
    /// Its file, vault-relative.
    pub path: String,
}

fn uid_of(id: &str) -> Result<Uid, StoreError> {
    parse_ulid(id).ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))
}

/// Hand out the next key for an item of `kind` (`stack`, `note`, `card` or `question`): one more than
/// the highest number in the index or ever handed out, which is remembered. `None` for other kinds.
pub fn allocate_key(conn: &Connection, kind: &str) -> Result<Option<String>, StoreError> {
    let Some(prefix) = key_prefix(kind) else {
        return Ok(None);
    };
    let counter: u32 = conn
        .query_row(
            "SELECT last FROM key_counter WHERE prefix = ?1",
            [prefix],
            |r| r.get(0),
        )
        .optional()
        .map_err(sql_err)?
        .unwrap_or(0);
    let mut stmt = conn
        .prepare_cached("SELECT key FROM item WHERE key LIKE ?1 || '\\_%' ESCAPE '\\'")
        .map_err(sql_err)?;
    let taken = stmt
        .query_map([prefix], |r| r.get::<_, String>(0))
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;
    let key = next_key(prefix, taken.iter().map(String::as_str), counter);
    let n = parse_key(&key).map_or(0, |(_, n)| n);
    conn.execute(
        "INSERT INTO key_counter (prefix, last) VALUES (?1, ?2)
         ON CONFLICT(prefix) DO UPDATE SET last = MAX(last, ?2)",
        params![prefix, n],
    )
    .map_err(sql_err)?;
    Ok(Some(key))
}

/// The item with this key, whatever case it is typed in; the first by path if several share it.
pub fn find_by_key(conn: &Connection, text: &str) -> Result<Option<KeyHit>, StoreError> {
    let Some(key) = normalize_key(text) else {
        return Ok(None);
    };
    let found = conn
        .query_row(
            "SELECT uid, kind, title, path FROM item WHERE key = ?1 COLLATE NOCASE ORDER BY path LIMIT 1",
            [key],
            |r| Ok((r.get::<_, String>(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()
        .map_err(sql_err)?;
    let Some((id, kind, title, path)) = found else {
        return Ok(None);
    };
    Ok(Some(KeyHit {
        uid: uid_of(&id)?,
        kind,
        title,
        path,
    }))
}

/// Every stack, note, card and question without a key, oldest first (by creation, then path), so
/// assigning keys numbers them in the order they were made.
pub fn items_without_key(conn: &Connection) -> Result<Vec<KeyHit>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT uid, kind, title, path FROM item WHERE key IS NULL
             ORDER BY kind, created_ms IS NULL, created_ms, path",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .map_err(sql_err)?;
    rows.map(|row| {
        let (id, kind, title, path) = row.map_err(sql_err)?;
        Ok(KeyHit {
            uid: uid_of(&id)?,
            kind,
            title,
            path,
        })
    })
    .collect()
}
