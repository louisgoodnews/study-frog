//! Changing many items at once from the library: status, priority, difficulty, tags, stack, delete.
//! Every call gives back a snapshot of the files it touched, so it can be undone as one step.

use std::fs;
use std::path::Path;

use studyfrog_core::event::{Difficulty, difficulty_name};
use studyfrog_core::priority::{Priority, priority_name};
use studyfrog_core::status::Status;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_md::extra::with_extra;
use studyfrog_md::render::render_doc;
use studyfrog_md::status::with_status;
use studyfrog_store::overview::load_overview;
use studyfrog_store::poll::prime;
use studyfrog_store::reindex::reindex;

use crate::create::write_new;
use crate::document::load_document;
use crate::session::{ServiceError, VaultInfo, VaultSession, vault_info};

/// What to do to every selected item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BulkEdit {
    /// Set the status.
    Status(Status),
    /// Set the priority.
    Priority(Priority),
    /// Set the difficulty.
    Difficulty(Difficulty),
    /// Add a tag.
    AddTag(String),
    /// Remove a tag.
    RemoveTag(String),
    /// Move to the stack with this folder.
    MoveTo(String),
    /// Delete the files.
    Delete,
}

/// The files a bulk edit touched as they were before: a vault-relative path and its text, `None` if the file did not exist.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Snapshot(pub Vec<(String, Option<String>)>);

/// What a bulk edit did.
#[derive(Debug, Clone, PartialEq)]
pub struct BulkDone {
    /// The items changed.
    pub changed: usize,
    /// The items left alone, each with the reason.
    pub skipped: Vec<String>,
    /// How to take it back.
    pub undo: Snapshot,
    /// The vault after reindexing.
    pub info: VaultInfo,
}

fn io(path: &str, e: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{path}: {e}"))
}

fn tags_of(doc: &mut Doc) -> &mut Vec<String> {
    match doc {
        Doc::Stack(d) => &mut d.meta.tags,
        Doc::Note(d) => &mut d.meta.tags,
        Doc::Card(d) => &mut d.meta.tags,
        Doc::Question(d) => &mut d.meta.tags,
    }
}

fn stamped(doc: &Doc, now: UnixMs) -> Doc {
    let mut doc = doc.clone();
    match &mut doc {
        Doc::Stack(d) => d.meta.updated = Some(now),
        Doc::Note(d) => d.meta.updated = Some(now),
        Doc::Card(d) => d.meta.updated = Some(now),
        Doc::Question(d) => d.meta.updated = Some(now),
    }
    doc
}

/// The document with a field edit applied; `None` for the edits that are not field edits.
fn edited(doc: &Doc, edit: &BulkEdit) -> Option<Doc> {
    let mut out = match edit {
        BulkEdit::Status(s) => with_status(doc, Some(*s)),
        BulkEdit::Priority(p) => with_extra(doc, "priority", Some(priority_name(*p))),
        BulkEdit::Difficulty(d) => with_extra(doc, "difficulty", Some(difficulty_name(*d))),
        BulkEdit::AddTag(_) | BulkEdit::RemoveTag(_) => doc.clone(),
        BulkEdit::MoveTo(_) | BulkEdit::Delete => return None,
    };
    match edit {
        BulkEdit::AddTag(t) if !tags_of(&mut out).contains(t) => tags_of(&mut out).push(t.clone()),
        BulkEdit::RemoveTag(t) => tags_of(&mut out).retain(|x| x != t),
        _ => {}
    }
    Some(out)
}

/// A free file name in `dir` for `name`: itself, or with `-2`, `-3`, ... before the extension.
fn free_name(dir: &Path, name: &str) -> String {
    let (stem, ext) = name.rsplit_once('.').map_or((name, ""), |(s, e)| (s, e));
    let dot = if ext.is_empty() { "" } else { "." };
    std::iter::once(name.to_owned())
        .chain((2..).map(|n| format!("{stem}-{n}{dot}{ext}")))
        .find(|n| !dir.join(n).exists())
        .unwrap_or_else(|| name.to_owned())
}

fn move_to(session: &VaultSession, path: &str, folder: &str) -> Result<String, ServiceError> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let dir = session.root.join(folder);
    fs::create_dir_all(&dir).map_err(|e| io(folder, &e))?;
    let target = format!("{folder}/{}", free_name(&dir, name));
    fs::rename(session.root.join(path), session.root.join(&target)).map_err(|e| io(path, &e))?;
    Ok(target)
}

/// Apply one edit to one item; the files it touched go to `undo`.
fn one(
    session: &VaultSession,
    (uid, edit, now): (Uid, &BulkEdit, UnixMs),
    undo: &mut Snapshot,
) -> Result<(), ServiceError> {
    let loaded = load_document(session, uid)?;
    if matches!(loaded.doc, Doc::Stack(_)) {
        return Err(ServiceError::Invalid(
            "a stack is not changed here".to_owned(),
        ));
    }
    let path = loaded.path.as_str();
    let before = fs::read_to_string(session.root.join(path)).map_err(|e| io(path, &e))?;
    match edit {
        BulkEdit::Delete => {
            undo.0.push((path.to_owned(), Some(before)));
            fs::remove_file(session.root.join(path)).map_err(|e| io(path, &e))
        }
        BulkEdit::MoveTo(folder) => {
            let target = move_to(session, path, folder)?;
            undo.0.push((path.to_owned(), Some(before)));
            undo.0.push((target, None));
            Ok(())
        }
        field => {
            let doc = edited(&loaded.doc, field).map(|d| stamped(&d, now));
            let text = doc
                .map_or_else(|| Ok(before.clone()), |d| render_doc(&d))
                .map_err(|e| ServiceError::Invalid(e.to_string()))?;
            undo.0.push((path.to_owned(), Some(before)));
            write_new(&session.root.join(path), &text)
        }
    }
}

fn refresh(session: &mut VaultSession) -> Result<VaultInfo, ServiceError> {
    reindex(&mut session.conn, &session.root).map_err(ServiceError::Store)?;
    session.poll = prime(&session.root).map_err(ServiceError::Store)?;
    vault_info(&session.root, &session.conn)
}

/// Apply `edit` to the items; the ones that cannot be changed are listed, the rest are done.
pub fn bulk_apply(
    session: &mut VaultSession,
    uids: &[Uid],
    edit: &BulkEdit,
    now: UnixMs,
) -> Result<BulkDone, ServiceError> {
    if let BulkEdit::MoveTo(folder) = edit {
        let stacks = load_overview(&session.conn)
            .map_err(ServiceError::Store)?
            .stacks;
        if !stacks.iter().any(|s| s.folder == *folder) {
            return Err(ServiceError::Invalid(format!(
                "there is no stack \"{folder}\""
            )));
        }
    }
    let (mut undo, mut skipped, mut changed) = (Snapshot::default(), Vec::new(), 0);
    for uid in uids {
        match one(session, (*uid, edit, now), &mut undo) {
            Ok(()) => changed += 1,
            Err(e) => skipped.push(e.to_string()),
        }
    }
    let info = refresh(session)?;
    Ok(BulkDone {
        changed,
        skipped,
        undo,
        info,
    })
}

/// Put the files of a snapshot back as they were, in reverse order.
pub fn bulk_undo(
    session: &mut VaultSession,
    snapshot: &Snapshot,
) -> Result<VaultInfo, ServiceError> {
    for (path, text) in snapshot.0.iter().rev() {
        let file = session.root.join(path);
        match text {
            Some(t) => write_new(&file, t)?,
            None => {
                if file.exists() {
                    fs::remove_file(&file).map_err(|e| io(path, &e))?;
                }
            }
        }
    }
    refresh(session)
}
