//! Where proposals of AI clients wait for the user (STORY-0119): one JSON file per proposal in the
//! vault's `.studyfrog/staged/` folder, next to the index and outside the notes. Staging writes
//! nothing else; applying goes through [`crate::agent_apply::apply_changes`] once the user accepts.

use std::fs;
use std::path::{Path, PathBuf};

use studyfrog_agent::mcp_stage::{StagedProposal, parse_staged, staged_json};
use studyfrog_agent::wiki::Change;
use studyfrog_md::ulid::{compose_ulid, format_ulid};
use studyfrog_store::overview::load_overview;

use crate::create::IdSource;
use crate::session::{ServiceError, VaultSession};

/// The folder of the staged proposals of the vault at `root`.
pub fn staged_dir(root: &Path) -> PathBuf {
    root.join(".studyfrog").join("staged")
}

/// A staged proposal as read now, with the changes that no longer pass the checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staged {
    /// The proposal (only its changes that still pass).
    pub proposal: StagedProposal,
    /// What was left out, each in a sentence.
    pub rejected: Vec<String>,
}

fn io(path: &Path, e: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{}: {e}", path.display()))
}

/// An id is a ULID: 26 letters and digits, so it is always a safe file name.
fn safe_id(id: &str) -> Result<&str, ServiceError> {
    if id.len() == 26 && id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Ok(id);
    }
    Err(ServiceError::Invalid(format!(
        "{id} is not a staged proposal"
    )))
}

/// Stage `changes` from `tool` with a summary; returns the new proposal's id.
pub fn stage_changes(
    root: &Path,
    (tool, summary): (&str, &str),
    changes: Vec<Change>,
    ids: IdSource,
) -> Result<String, ServiceError> {
    let id = format_ulid(compose_ulid(ids.now_ms, ids.random));
    let proposal = StagedProposal {
        id: id.clone(),
        created_ms: i64::try_from(ids.now_ms).unwrap_or(i64::MAX),
        tool: tool.to_owned(),
        summary: summary.to_owned(),
        changes,
    };
    let dir = staged_dir(root);
    fs::create_dir_all(&dir).map_err(|e| io(&dir, &e))?;
    let (target, temp) = (
        dir.join(format!("{id}.json")),
        dir.join(format!("{id}.tmp")),
    );
    fs::write(&temp, staged_json(&proposal)).map_err(|e| io(&temp, &e))?;
    fs::rename(&temp, &target).map_err(|e| io(&target, &e))?;
    Ok(id)
}

/// Every staged proposal of the vault, oldest first, each checked against the stacks as they are now.
/// A file that cannot be read is skipped.
pub fn list_staged(session: &VaultSession) -> Result<Vec<Staged>, ServiceError> {
    let stacks: Vec<String> = load_overview(&session.conn)
        .map_err(ServiceError::Store)?
        .stacks
        .into_iter()
        .map(|s| s.folder)
        .collect();
    let Ok(entries) = fs::read_dir(staged_dir(&session.root)) else {
        return Ok(Vec::new());
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    Ok(paths
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .filter_map(|text| parse_staged(&text, &stacks).ok())
        .map(|(proposal, rejected)| Staged { proposal, rejected })
        .collect())
}

/// Remove a staged proposal (after it was applied or when the user discards it).
pub fn remove_staged(root: &Path, id: &str) -> Result<(), ServiceError> {
    let path = staged_dir(root).join(format!("{}.json", safe_id(id)?));
    match fs::remove_file(&path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(io(&path, &e)),
        _ => Ok(()),
    }
}
