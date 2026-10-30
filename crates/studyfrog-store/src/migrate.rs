//! Ordered, transactional migrations tracked with `PRAGMA user_version`.

use rusqlite::Connection;

use crate::error::{StoreError, sql_err};

/// One schema step. Versions are consecutive integers starting at 1.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    /// Target `user_version` after this step.
    pub version: u32,
    /// Short description.
    pub name: &'static str,
    /// SQL script (may contain several statements).
    pub sql: &'static str,
}

/// Current `user_version` of the database (0 for a fresh file).
pub fn schema_version(conn: &Connection) -> Result<u32, StoreError> {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(sql_err)
}

fn apply(conn: &mut Connection, step: &Migration) -> Result<(), StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    tx.execute_batch(step.sql).map_err(sql_err)?;
    tx.pragma_update(None, "user_version", step.version)
        .map_err(sql_err)?;
    tx.commit().map_err(sql_err)
}

/// Bring the database up to the last migration; returns the resulting version.
/// A database newer than `migrations` is refused untouched.
pub fn migrate(conn: &mut Connection, migrations: &[Migration]) -> Result<u32, StoreError> {
    let current = schema_version(conn)?;
    let latest = migrations.last().map_or(0, |m| m.version);
    if current > latest {
        return Err(StoreError::SchemaTooNew {
            found: current,
            supported: latest,
        });
    }
    for step in migrations.iter().filter(|m| m.version > current) {
        apply(conn, step)?;
    }
    schema_version(conn)
}
