//! Store errors. `Display` delegates to a free function (P1).

use std::fmt;

/// What can go wrong in the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// SQLite reported an error; carries its message.
    Sql(String),
    /// A file-system operation failed.
    Io {
        /// The path involved.
        path: String,
        /// The OS message.
        message: String,
    },
    /// Stored data does not decode (for example an unknown name or a malformed id).
    Corrupt(String),
    /// A settings file is invalid.
    Config {
        /// The file.
        path: String,
        /// What is wrong.
        message: String,
    },
    /// The database was written by a newer version of the app.
    SchemaTooNew {
        /// Version found in the database.
        found: u32,
        /// Highest version this build understands.
        supported: u32,
    },
}

/// Build an [`StoreError::Sql`] from a rusqlite error.
pub fn sql_err(e: rusqlite::Error) -> StoreError {
    StoreError::Sql(e.to_string())
}

/// Build an [`StoreError::Io`].
pub fn io_err(path: &std::path::Path, e: &std::io::Error) -> StoreError {
    StoreError::Io {
        path: path.display().to_string(),
        message: e.to_string(),
    }
}

/// Human-readable description.
pub fn describe_store_error(e: &StoreError) -> String {
    match e {
        StoreError::Sql(m) => format!("database error: {m}"),
        StoreError::Corrupt(m) => format!("corrupt stored data: {m}"),
        StoreError::Config { path, message } => format!("{path}: {message}"),
        StoreError::Io { path, message } => format!("{path}: {message}"),
        StoreError::SchemaTooNew { found, supported } => {
            format!("database schema v{found} is newer than this app supports (v{supported})")
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&describe_store_error(self))
    }
}

impl std::error::Error for StoreError {}
