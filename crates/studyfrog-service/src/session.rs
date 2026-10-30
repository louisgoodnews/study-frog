//! An open vault: opening it (index, migrate, reindex) and keeping it current by polling.

use std::fmt;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use studyfrog_core::settings::{VaultSettings, vault_settings_default};
use studyfrog_store::db::open_db;
use studyfrog_store::error::StoreError;
use studyfrog_store::migrate::migrate;
use studyfrog_store::overview::{VaultOverview, load_overview};
use studyfrog_store::poll::{PollOutcome, PollState, poll_changes, prime};
use studyfrog_store::reindex::reindex;
use studyfrog_store::schema::MIGRATIONS;
use studyfrog_store::vault_config::read_vault_settings;

/// Files modified more recently than this are left for the next poll (half-written files).
pub const SETTLE_NS: i64 = 200_000_000;
/// Where the index lives inside a vault (hidden, so it is never scanned as content).
const INDEX_DIR: &str = ".studyfrog";
const INDEX_FILE: &str = "index.sqlite";

/// Why a vault could not be opened or checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    /// The path is not a folder.
    NotAFolder(String),
    /// The index folder could not be created.
    Io(String),
    /// The index failed.
    Store(StoreError),
    /// The request itself is not acceptable (empty title, not a stack, ...).
    Invalid(String),
}

/// Human-readable description.
pub fn describe_service_error(e: &ServiceError) -> String {
    match e {
        ServiceError::NotAFolder(p) => format!("{p} is not a folder"),
        ServiceError::Io(m) | ServiceError::Invalid(m) => m.clone(),
        ServiceError::Store(e) => e.to_string(),
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&describe_service_error(self))
    }
}

impl std::error::Error for ServiceError {}

/// An open vault and its index connection.
pub struct VaultSession {
    /// Vault folder.
    pub root: PathBuf,
    /// Connection to the vault's index.
    pub conn: Connection,
    /// Change-detection state.
    pub poll: PollState,
}

/// What a front end shows about an open vault.
#[derive(Debug, Clone, PartialEq)]
pub struct VaultInfo {
    /// Absolute path of the vault folder.
    pub root: String,
    /// Folder name, used as the title.
    pub name: String,
    /// Stacks, inbox count and problems.
    pub overview: VaultOverview,
    /// Settings from `studyfrog.toml` (defaults when absent or invalid).
    pub settings: VaultSettings,
    /// Why `studyfrog.toml` was not used, if it was invalid.
    pub settings_error: Option<String>,
}

fn store(e: StoreError) -> ServiceError {
    ServiceError::Store(e)
}

/// Overview and effective settings of the vault, read from the index and `studyfrog.toml`.
pub fn vault_info(root: &Path, conn: &Connection) -> Result<VaultInfo, ServiceError> {
    let overview = load_overview(conn).map_err(store)?;
    let (settings, settings_error) = match read_vault_settings(root) {
        Ok(s) => (s, None),
        Err(e) => (vault_settings_default(), Some(e.to_string())),
    };
    let name = root.file_name().map_or_else(
        || root.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    Ok(VaultInfo {
        root: root.display().to_string(),
        name,
        overview,
        settings,
        settings_error,
    })
}

/// Open a vault folder: create its index if needed, bring the index up to date, and describe it.
pub fn open_vault(path: &str) -> Result<(VaultSession, VaultInfo), ServiceError> {
    let root = PathBuf::from(path);
    if !root.is_dir() {
        return Err(ServiceError::NotAFolder(path.to_owned()));
    }
    let dir = root.join(INDEX_DIR);
    std::fs::create_dir_all(&dir)
        .map_err(|e| ServiceError::Io(format!("cannot create {}: {e}", dir.display())))?;
    let mut conn = open_db(&dir.join(INDEX_FILE)).map_err(store)?;
    migrate(&mut conn, MIGRATIONS).map_err(store)?;
    // Fingerprint first: a file that changes while indexing still shows up at the next poll.
    let poll = prime(&root).map_err(store)?;
    reindex(&mut conn, &root).map_err(store)?;
    let info = vault_info(&root, &conn)?;
    Ok((VaultSession { root, conn, poll }, info))
}

/// Check the vault for changes. `Some(info)` when something was reindexed, `None` otherwise.
pub fn poll_vault(
    session: &mut VaultSession,
    now_ns: i64,
) -> Result<Option<VaultInfo>, ServiceError> {
    let (state, outcome) = poll_changes(
        &mut session.conn,
        &session.root,
        session.poll,
        now_ns,
        SETTLE_NS,
    )
    .map_err(store)?;
    session.poll = state;
    match outcome {
        PollOutcome::Reindexed(_) => vault_info(&session.root, &session.conn).map(Some),
        PollOutcome::Unchanged | PollOutcome::Settling => Ok(None),
    }
}
