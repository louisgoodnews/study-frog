//! The goals of a stack, kept under a `## Goals` heading in its `_stack.md`.

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_md::goals::{goals_of, goals_problem, with_goals};

use crate::document::{load_document, save_document};
use crate::session::{ServiceError, VaultInfo, VaultSession};

fn stack_body(doc: &Doc) -> Result<&str, ServiceError> {
    match doc {
        Doc::Stack(d) => Ok(&d.body),
        _ => Err(ServiceError::Invalid("only a stack has goals".to_owned())),
    }
}

/// The goals of the stack `uid`.
pub fn load_goals(session: &VaultSession, uid: Uid) -> Result<String, ServiceError> {
    let loaded = load_document(session, uid)?;
    Ok(goals_of(stack_body(&loaded.doc)?))
}

/// Replace the goals of the stack `uid`; the rest of the file is kept.
pub fn save_goals(
    session: &mut VaultSession,
    uid: Uid,
    text: &str,
    now: UnixMs,
) -> Result<VaultInfo, ServiceError> {
    if let Some(why) = goals_problem(text) {
        return Err(ServiceError::Invalid(why.to_owned()));
    }
    let loaded = load_document(session, uid)?;
    let body = with_goals(stack_body(&loaded.doc)?, text);
    let Doc::Stack(mut stack) = loaded.doc else {
        return Err(ServiceError::Invalid("only a stack has goals".to_owned()));
    };
    stack.body = body;
    save_document(session, &loaded.path, &Doc::Stack(stack), loaded.hash, now).map(|s| s.info)
}
