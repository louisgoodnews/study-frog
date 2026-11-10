//! Item comments in front-matter (STORY-0135, STORY-0140).
//!
//! ```yaml
//! comments:
//!   - id: c_123
//!     author: user
//!     created: 1731140000000
//!     text: "Hint or note"
//! ```

use studyfrog_core::comment::ItemComment;

use crate::doc::{Doc, RawEntry};

/// The front-matter key for comments.
pub const COMMENTS_FIELD: &str = "comments";

#[derive(serde::Serialize, serde::Deserialize)]
struct RawCommentsWrapper {
    comments: Vec<RawComment>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RawComment {
    id: String,
    author: String,
    created: i64,
    text: String,
}

fn entries(doc: &Doc) -> &[RawEntry] {
    match doc {
        Doc::Stack(d) => &d.extra,
        Doc::Note(d) => &d.extra,
        Doc::Card(d) => &d.extra,
        Doc::Question(d) => &d.extra,
        Doc::Diary(d) => &d.extra,
    }
}

fn entries_mut(doc: &mut Doc) -> &mut Vec<RawEntry> {
    match doc {
        Doc::Stack(d) => &mut d.extra,
        Doc::Note(d) => &mut d.extra,
        Doc::Card(d) => &mut d.extra,
        Doc::Question(d) => &mut d.extra,
        Doc::Diary(d) => &mut d.extra,
    }
}

/// The comments of a document, if any.
pub fn comments_of(doc: &Doc) -> Vec<ItemComment> {
    let Some(entry) = entries(doc).iter().find(|e| e.key == COMMENTS_FIELD) else {
        return Vec::new();
    };
    let parsed: Result<RawCommentsWrapper, _> = serde_saphyr::from_str(&entry.text);
    match parsed {
        Ok(wrapper) => wrapper
            .comments
            .into_iter()
            .map(|c| ItemComment {
                id: c.id,
                author: c.author,
                created: c.created,
                text: c.text,
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Replace or set the comments of a document.
pub fn with_comments(doc: &Doc, comments: &[ItemComment]) -> Doc {
    let mut out = doc.clone();
    let list = entries_mut(&mut out);
    list.retain(|e| e.key != COMMENTS_FIELD);
    if !comments.is_empty() {
        let raw = RawCommentsWrapper {
            comments: comments
                .iter()
                .map(|c| RawComment {
                    id: c.id.clone(),
                    author: c.author.clone(),
                    created: c.created,
                    text: c.text.clone(),
                })
                .collect(),
        };
        if let Ok(text) = serde_saphyr::to_string(&raw) {
            list.push(RawEntry {
                key: COMMENTS_FIELD.to_owned(),
                text: text.trim().to_owned(),
            });
        }
    }
    out
}
