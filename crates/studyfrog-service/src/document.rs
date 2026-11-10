//! Editing a document: load it with a fingerprint, save it only if the file is unchanged since.

use std::fs;

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{Doc, doc_meta};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;
use studyfrog_store::hash::hash_bytes;
use studyfrog_store::overview::{item_path, resolve_title};
use studyfrog_store::poll::prime;
use studyfrog_store::reindex::reindex;

use crate::session::{ServiceError, VaultInfo, VaultSession, vault_info};

/// A parsed file together with what is needed to save it safely.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedDoc {
    /// Vault-relative path.
    pub path: String,
    /// The parsed document (unknown front-matter keys included).
    pub doc: Doc,
    /// Hash of the bytes that were read.
    pub hash: u64,
}

/// A successful save: the refreshed vault and the new fingerprint of the file.
#[derive(Debug, Clone, PartialEq)]
pub struct Saved {
    /// The vault after reindexing.
    pub info: VaultInfo,
    /// Hash of the bytes written.
    pub hash: u64,
}

fn io(path: &str, e: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{path}: {e}"))
}

/// Read and parse an item's file; the path comes from the index.
pub fn load_document(session: &VaultSession, uid: Uid) -> Result<LoadedDoc, ServiceError> {
    let path = item_path(&session.conn, uid)
        .map_err(ServiceError::Store)?
        .ok_or_else(|| ServiceError::Invalid("that item is no longer in the vault".to_owned()))?;
    let bytes = fs::read(session.root.join(&path)).map_err(|e| io(&path, &e))?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| ServiceError::Invalid(format!("{path} is not valid UTF-8")))?;
    let doc = parse_doc(text).map_err(|e| ServiceError::Invalid(format!("{path}: {e}")))?;
    Ok(LoadedDoc {
        path,
        doc,
        hash: hash_bytes(&bytes),
    })
}

/// Save `doc` over `path`. Refused with [`ServiceError::Conflict`] if the file no longer has the
/// content that was loaded (`expected_hash`), so edits made elsewhere are never overwritten. The
/// document's `updated` time is set to `now`. The write goes through a temporary file.
pub fn save_document(
    session: &mut VaultSession,
    path: &str,
    doc: &Doc,
    expected_hash: u64,
    now: UnixMs,
) -> Result<Saved, ServiceError> {
    let target = session.root.join(path);
    let current = fs::read(&target).map_err(|e| io(path, &e))?;
    if hash_bytes(&current) != expected_hash {
        return Err(ServiceError::Conflict(format!(
            "{path} was changed outside StudyFrog since you opened it"
        )));
    }
    let text =
        render_doc(&with_updated(doc, now)).map_err(|e| ServiceError::Invalid(e.to_string()))?;
    let temp = target.with_extension("md.tmp");
    fs::write(&temp, &text).map_err(|e| io(path, &e))?;
    fs::rename(&temp, &target).map_err(|e| io(path, &e))?;
    reindex(&mut session.conn, &session.root).map_err(ServiceError::Store)?;
    session.poll = prime(&session.root).map_err(ServiceError::Store)?;
    Ok(Saved {
        info: vault_info(&session.root, &session.conn)?,
        hash: hash_bytes(text.as_bytes()),
    })
}

fn with_updated(doc: &Doc, now: UnixMs) -> Doc {
    let mut doc = doc.clone();
    match &mut doc {
        Doc::Stack(d) => d.meta.updated = Some(now),
        Doc::Note(d) => d.meta.updated = Some(now),
        Doc::Card(d) => d.meta.updated = Some(now),
        Doc::Question(d) => d.meta.updated = Some(now),
        Doc::Diary(d) => d.meta.updated = Some(now),
    }
    debug_assert_eq!(doc_meta(&doc).updated, Some(now));
    doc
}

/// The item a wikilink points to, by title (case-insensitive).
pub fn resolve_link(session: &VaultSession, target: &str) -> Result<Option<Uid>, ServiceError> {
    resolve_title(&session.conn, target).map_err(ServiceError::Store)
}

/// Append a comment to an item document and save it.
pub fn append_comment(
    session: &mut VaultSession,
    uid: Uid,
    comment: studyfrog_core::comment::ItemComment,
    now: UnixMs,
) -> Result<(), ServiceError> {
    let loaded = load_document(session, uid)?;
    let mut comments = studyfrog_md::comments::comments_of(&loaded.doc);
    comments.push(comment);
    let updated = studyfrog_md::comments::with_comments(&loaded.doc, &comments);
    save_document(session, &loaded.path, &updated, loaded.hash, now)?;
    Ok(())
}
