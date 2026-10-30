//! What the rehearsal screen shows for an item, read from its file on demand.

use std::fs;

use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{Doc, doc_meta};
use studyfrog_md::parse::parse_doc;
use studyfrog_store::overview::item_path;

use crate::session::{ServiceError, VaultSession};

/// An item reduced to what is asked and what is answered, trimmed for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemContent {
    /// The item's title.
    pub title: String,
    /// What is shown first: a card's front, a question's prompt, a note's title.
    pub prompt: String,
    /// What is revealed: a card's back, a question's reference answer, a note's body.
    pub answer: String,
}

fn content_of(doc: &Doc) -> ItemContent {
    let title = doc_meta(doc).title.clone();
    let shown = |text: &str| text.trim_end().to_owned();
    match doc {
        Doc::Card(c) => ItemContent {
            title,
            prompt: shown(&c.front),
            answer: shown(&c.back),
        },
        Doc::Question(q) => ItemContent {
            title,
            prompt: shown(&q.prompt),
            answer: shown(&q.reference),
        },
        Doc::Note(n) => ItemContent {
            prompt: title.clone(),
            title,
            answer: shown(&n.body),
        },
        Doc::Stack(s) => ItemContent {
            prompt: title.clone(),
            title,
            answer: shown(&s.description),
        },
    }
}

/// Read and parse the item's file. The path comes from the index, so a renamed file is found.
pub fn load_item_content(session: &VaultSession, uid: Uid) -> Result<ItemContent, ServiceError> {
    let rel = item_path(&session.conn, uid)
        .map_err(ServiceError::Store)?
        .ok_or_else(|| ServiceError::Invalid("that item is no longer in the vault".to_owned()))?;
    let path = session.root.join(&rel);
    let text = fs::read_to_string(&path).map_err(|e| ServiceError::Io(format!("{rel}: {e}")))?;
    let doc = parse_doc(&text).map_err(|e| ServiceError::Invalid(format!("{rel}: {e}")))?;
    Ok(content_of(&doc))
}
