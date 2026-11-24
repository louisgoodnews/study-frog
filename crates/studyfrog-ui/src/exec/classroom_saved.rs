//! Saved classroom sessions on the worker (TASK-0295): the browser's list, resume, Markdown export
//! and delete. Each reads or writes the vault's store and answers with one message to the screen.

use std::fs;

use studyfrog_core::classroom::ClassroomSession;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::session::VaultSession;
use studyfrog_store::classroom_store::{
    delete_session, export_session_markdown, list_sessions, load_session,
};

use crate::classroom_types::{ClassroomMsg, SavedEffect, SavedMsg, SavedSession};
use crate::exec::Worker;
use crate::msg::Msg;

const NOT_SAVED: &str = "That session is no longer saved.";

/// Runs one saved-session effect with the open vault.
pub fn run(w: &Worker, effect: SavedEffect) -> Msg {
    let Some(vault) = w.session.as_ref() else {
        return listed(Vec::new(), None, Some("Open a vault first.".to_owned()));
    };
    match effect {
        SavedEffect::Load { id } => load(vault, id),
        SavedEffect::List => finish(vault, Ok(None)),
        SavedEffect::Export { id } => finish(vault, export(vault, id)),
        SavedEffect::Delete { id } => finish(vault, delete(vault, id)),
    }
}

/// Loads the session onto the screen, as the classroom screen's own `Done` message.
fn load(vault: &VaultSession, id: Uid) -> Msg {
    let (session, error) = match load_session(&vault.conn, id) {
        Ok(Some(s)) => (Some(s), None),
        Ok(None) => (None, Some(NOT_SAVED.to_owned())),
        Err(e) => (None, Some(e.to_string())),
    };
    Msg::Classroom(ClassroomMsg::Done { session, error })
}

/// Reads the list back after an action, and reports what the action did or why it failed.
fn finish(vault: &VaultSession, outcome: Result<Option<String>, String>) -> Msg {
    let (notice, error) = match outcome {
        Ok(notice) => (notice, None),
        Err(e) => (None, Some(e)),
    };
    match stored(vault) {
        Ok(sessions) => listed(sessions, notice, error),
        Err(e) => listed(Vec::new(), notice, error.or(Some(e))),
    }
}

fn listed(sessions: Vec<SavedSession>, notice: Option<String>, error: Option<String>) -> Msg {
    Msg::Classroom(ClassroomMsg::Saved(SavedMsg::Listed {
        sessions,
        notice,
        error,
    }))
}

/// Every saved session, newest first, as the browser lists them.
fn stored(vault: &VaultSession) -> Result<Vec<SavedSession>, String> {
    let all = list_sessions(&vault.conn).map_err(|e| e.to_string())?;
    Ok(all.iter().map(saved_of).collect())
}

/// The browser's row for one stored session.
pub fn saved_of(s: &ClassroomSession) -> SavedSession {
    SavedSession {
        id: s.id,
        title: s.title.clone(),
        created_at: s.created_at,
        sources: s.sources.clone(),
        turns: s.turns.len(),
        tasks: s.assignments.len(),
    }
}

/// Writes the transcript into the vault's `exports` folder as `classroom-<id>.md`; the path reported.
fn export(vault: &VaultSession, id: Uid) -> Result<Option<String>, String> {
    let text = export_session_markdown(&vault.conn, id)
        .map_err(|e| e.to_string())?
        .ok_or(NOT_SAVED)?;
    let dir = vault.root.join("exports");
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let target = dir.join(format!("classroom-{}.md", format_ulid(id)));
    fs::write(&target, text).map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(Some(format!("Exported to {}", target.display())))
}

/// Deletes the session and its rows. A session that is already gone is reported, not ignored.
fn delete(vault: &VaultSession, id: Uid) -> Result<Option<String>, String> {
    if delete_session(&vault.conn, id).map_err(|e| e.to_string())? {
        Ok(Some("Session deleted.".to_owned()))
    } else {
        Err(NOT_SAVED.to_owned())
    }
}
