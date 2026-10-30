//! Change detection by polling. Cheaper and simpler than a watcher dependency (P10): one poll
//! stats the vault (about 43 ms for 10k files, see BENCHMARKING.md) and reindexes only when the
//! stat fingerprint moved. A burst of writes is coalesced into one reindex, and files modified
//! within the settle window are left for the next poll so half-written files are not indexed.
//! The clock is an argument; nothing here reads the time.

use std::path::Path;

use rusqlite::Connection;

use crate::error::StoreError;
use crate::hash::{combine, hash_bytes};
use crate::reindex::{ReindexReport, reindex_files};
use crate::vault::{VaultFile, list_markdown};

/// Hash of every file's path, size and mtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fingerprint(pub u64);

/// What the poller remembers between polls. Start with `PollState::default()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PollState {
    /// Fingerprint at the last successful reindex; `None` before the first one.
    pub indexed: Option<Fingerprint>,
}

/// Result of one poll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollOutcome {
    /// Nothing changed since the last reindex.
    Unchanged,
    /// Something changed very recently; try again on the next poll.
    Settling,
    /// The vault changed and was reindexed.
    Reindexed(ReindexReport),
}

/// Fingerprint of a listing.
pub fn fingerprint(files: &[VaultFile]) -> Fingerprint {
    let h = files.iter().fold(files.len() as u64, |h, f| {
        let stat = f.size ^ (f.mtime_ns as u64).rotate_left(17);
        combine(combine(h, hash_bytes(f.rel.as_bytes())), stat)
    });
    Fingerprint(h)
}

fn newest_mtime(files: &[VaultFile]) -> i64 {
    files.iter().map(|f| f.mtime_ns).max().unwrap_or(0)
}

/// Check the vault once. `now_ns` is the current time in nanoseconds since the Unix epoch and
/// `settle_ns` how long a file must have been quiet before it is indexed.
pub fn poll_changes(
    conn: &mut Connection,
    root: &Path,
    state: PollState,
    now_ns: i64,
    settle_ns: i64,
) -> Result<(PollState, PollOutcome), StoreError> {
    let listed = list_markdown(root)?;
    let current = fingerprint(&listed);
    if state.indexed == Some(current) {
        return Ok((state, PollOutcome::Unchanged));
    }
    if now_ns.saturating_sub(newest_mtime(&listed)) < settle_ns {
        return Ok((state, PollOutcome::Settling));
    }
    let report = reindex_files(conn, &listed)?;
    Ok((
        PollState {
            indexed: Some(current),
        },
        PollOutcome::Reindexed(report),
    ))
}
