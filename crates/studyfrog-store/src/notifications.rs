//! The notifications of a vault: what the bell list shows, kept until the user deletes them or they age out.

use rusqlite::{Connection, params};

use crate::error::{StoreError, sql_err};

/// One stored notification. `kind` is `info` or `error`; `target` is free text the app understands
/// (`screen:calendar`, `item:<id>`), `None` when there is nothing to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredNotification {
    /// Identity, chosen by the caller and unique within the vault.
    pub id: i64,
    /// `info` or `error`.
    pub kind: String,
    /// The message.
    pub text: String,
    /// When it was raised, in Unix milliseconds.
    pub at_ms: i64,
    /// The user has dealt with it.
    pub read: bool,
    /// What a click opens.
    pub target: Option<String>,
}

/// Store a notification, replacing one with the same id.
pub fn insert_notification(conn: &Connection, n: &StoredNotification) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR REPLACE INTO notification (id, kind, text, at_ms, read, target) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![n.id, n.kind, n.text, n.at_ms, n.read, n.target],
    )
    .map(drop)
    .map_err(sql_err)
}

/// Every notification, oldest first.
pub fn load_notifications(conn: &Connection) -> Result<Vec<StoredNotification>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT id, kind, text, at_ms, read, target FROM notification ORDER BY at_ms, id")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok(StoredNotification {
                id: r.get(0)?,
                kind: r.get(1)?,
                text: r.get(2)?,
                at_ms: r.get(3)?,
                read: r.get(4)?,
                target: r.get(5)?,
            })
        })
        .map_err(sql_err)?;
    rows.collect::<Result<_, _>>().map_err(sql_err)
}

/// Mark one notification read or unread.
pub fn set_notification_read(conn: &Connection, id: i64, read: bool) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE notification SET read = ?2 WHERE id = ?1",
        params![id, read],
    )
    .map(drop)
    .map_err(sql_err)
}

/// Mark every notification read or unread.
pub fn set_all_notifications_read(conn: &Connection, read: bool) -> Result<(), StoreError> {
    conn.execute("UPDATE notification SET read = ?1", [read])
        .map(drop)
        .map_err(sql_err)
}

/// Delete one notification.
pub fn delete_notification(conn: &Connection, id: i64) -> Result<(), StoreError> {
    conn.execute("DELETE FROM notification WHERE id = ?1", [id])
        .map(drop)
        .map_err(sql_err)
}

/// Delete every notification.
pub fn clear_notifications(conn: &Connection) -> Result<(), StoreError> {
    conn.execute("DELETE FROM notification", [])
        .map(drop)
        .map_err(sql_err)
}

/// Keep at most `max_count` of the newest and drop everything raised before `oldest_ms`.
pub fn prune_notifications(
    conn: &Connection,
    max_count: usize,
    oldest_ms: i64,
) -> Result<(), StoreError> {
    conn.execute("DELETE FROM notification WHERE at_ms < ?1", [oldest_ms])
        .map_err(sql_err)?;
    conn.execute(
        "DELETE FROM notification WHERE id NOT IN (SELECT id FROM notification ORDER BY at_ms DESC, id DESC LIMIT ?1)",
        [max_count as i64],
    )
    .map(drop)
    .map_err(sql_err)
}
