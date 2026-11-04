//! Applying the changes of a proposal the user accepted.

use studyfrog_agent::wiki::{Change, PDF_PREFIX};
use studyfrog_core::time::UnixMs;
use studyfrog_md::doc::Doc;
use studyfrog_md::ulid::parse_ulid;

use crate::create::IdSource;
use crate::create_more::{NewItem, NewKind, create_item};
use crate::document::{load_document, save_document};
use crate::session::{ServiceError, VaultSession};

/// What applying a proposal did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Applied {
    /// New items.
    pub created: u32,
    /// Items changed.
    pub updated: u32,
    /// What could not be done, each in a sentence.
    pub failed: Vec<String>,
}

fn nth(ids: IdSource, n: usize) -> IdSource {
    let [hi, lo] = u16::try_from(n).unwrap_or(u16::MAX).to_be_bytes();
    let mut random = ids.random;
    random[5] ^= hi;
    random[6] ^= lo;
    IdSource { random, ..ids }
}

fn set_field(doc: &mut Doc, field: &str, text: &str) -> bool {
    match (doc, field) {
        (Doc::Note(d), "body") => d.body = text.to_owned(),
        (Doc::Card(d), "front") => d.front = text.to_owned(),
        (Doc::Card(d), "back") => d.back = text.to_owned(),
        (Doc::Question(d), "prompt") => d.prompt = text.to_owned(),
        (Doc::Question(d), "reference") => d.reference = text.to_owned(),
        (Doc::Stack(d), "body") => d.body = text.to_owned(),
        (Doc::Diary(d), "body") => d.body = text.to_owned(),
        _ => return false,
    }
    true
}

fn update(
    session: &mut VaultSession,
    uid: &str,
    fields: &[(String, String)],
    now: UnixMs,
) -> Result<(), ServiceError> {
    let id =
        parse_ulid(uid).ok_or_else(|| ServiceError::Invalid(format!("{uid} is not an item id")))?;
    let loaded = load_document(session, id)?;
    let mut doc = loaded.doc.clone();
    for (field, text) in fields {
        if !set_field(&mut doc, field, text) {
            return Err(ServiceError::Invalid(format!(
                "an item of this kind has no \"{field}\""
            )));
        }
    }
    save_document(session, &loaded.path, &doc, loaded.hash, now).map(|_| ())
}

fn create(session: &mut VaultSession, change: &Change, ids: IdSource) -> Result<(), ServiceError> {
    let (kind, stack, title, text, answer, from) = match change {
        Change::CreateNote { stack, title, body } => (
            NewKind::Note,
            stack,
            title.as_str(),
            body.as_str(),
            "",
            None,
        ),
        Change::CreateCard {
            stack,
            front,
            back,
            source,
        } => (
            NewKind::Card,
            stack,
            "",
            front.as_str(),
            back.as_str(),
            source.as_deref(),
        ),
        Change::CreateQuestion {
            stack,
            prompt,
            reference,
            source,
        } => (
            NewKind::Question,
            stack,
            "",
            prompt.as_str(),
            reference.as_str(),
            source.as_deref(),
        ),
        Change::UpdateItem { .. } | Change::CreatePage { .. } | Change::UpdatePage { .. } => {
            return Ok(());
        }
    };
    let source = from.and_then(|s| s.strip_prefix(PDF_PREFIX));
    let item = NewItem {
        source,
        kind,
        title,
        text,
        answer,
        tags: &[],
        parent: Some(stack.as_str()),
        priority: None,
        difficulty: None,
        also_in: None,
    };
    create_item(session, &item, ids).map(|_| ())
}

/// Apply the changes the user accepted, one by one; a failure is reported and the others still run.
pub fn apply_changes(
    session: &mut VaultSession,
    changes: &[Change],
    ids: IdSource,
    now: UnixMs,
) -> Applied {
    let mut out = Applied::default();
    for (i, change) in changes.iter().enumerate() {
        let result = match change {
            Change::UpdateItem { uid, fields } => {
                update(session, uid, fields, now).map(|()| out.updated += 1)
            }
            Change::CreatePage { .. } => {
                crate::agent_help::apply_page(&session.root, change).map(|()| out.created += 1)
            }
            Change::UpdatePage { .. } => {
                crate::agent_help::apply_page(&session.root, change).map(|()| out.updated += 1)
            }
            other => create(session, other, nth(ids, i)).map(|()| out.created += 1),
        };
        if let Err(e) = result {
            out.failed.push(format!("Change {}: {e}", i + 1));
        }
    }
    out
}
