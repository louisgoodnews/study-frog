//! Opening connections with the project's pragmas.

use std::path::Path;

use rusqlite::Connection;

use crate::error::{StoreError, sql_err};

const PRAGMAS: &str = "PRAGMA foreign_keys = ON; PRAGMA synchronous = NORMAL;";

/// Open (or create) the database file: WAL journal, foreign keys on.
pub fn open_db(path: &Path) -> Result<Connection, StoreError> {
    let conn = Connection::open(path).map_err(sql_err)?;
    conn.execute_batch(&format!("PRAGMA journal_mode = WAL; {PRAGMAS}"))
        .map_err(sql_err)?;
    Ok(conn)
}

/// An in-memory database with the same pragmas (no WAL), for tests.
pub fn open_memory() -> Result<Connection, StoreError> {
    let conn = Connection::open_in_memory().map_err(sql_err)?;
    conn.execute_batch(PRAGMAS).map_err(sql_err)?;
    Ok(conn)
}
