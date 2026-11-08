//! Cards and questions written inline in a note, synced into the note's stack (STORY-0110,
//! `docs/specs/inline-extraction.md`). Each item made from a block carries `extracted_from` (the
//! note's id) and `extract_key` (the block's key), so a second sync updates it instead of adding a
//! copy, and its answer ends with a `Source: [[Note#Heading]]` link back to the note.

use std::fs;

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{Doc, doc_meta};
use studyfrog_md::extra::extra_value;
use studyfrog_md::inline_extract::{
    InlineBlock, InlineKind, extract_inline, extract_key, source_target,
};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::ulid::format_ulid;

use crate::document::load_document;
use crate::session::{ServiceError, VaultSession};

/// The front-matter key naming the note an item was extracted from.
pub const FROM_FIELD: &str = "extracted_from";
/// The front-matter key holding the block's key.
pub const KEY_FIELD: &str = "extract_key";

/// What a sync does with one block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncChange {
    /// No item has this block yet: one is created.
    New,
    /// The item exists with another answer: its answer is replaced.
    Updated,
    /// The item exists and is current.
    Unchanged,
}

/// One block of the note and what the sync does with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncEntry {
    /// The block.
    pub block: InlineBlock,
    /// What happens.
    pub change: SyncChange,
    /// The item that has it, with its vault-relative path.
    pub existing: Option<(Uid, String)>,
    /// The answer the item gets (the block's answer and the source line).
    pub answer: String,
}

/// What a sync of a note would do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPlan {
    /// The note.
    pub note: Uid,
    /// Its title.
    pub title: String,
    /// The stack folder the items go into.
    pub folder: String,
    /// The note's tags, given to new items.
    pub tags: Vec<String>,
    /// One entry per block, in the note's order.
    pub entries: Vec<SyncEntry>,
}

/// How many entries of the plan have this change.
pub fn count_of(plan: &SyncPlan, change: SyncChange) -> usize {
    plan.entries.iter().filter(|e| e.change == change).count()
}

/// The answer an extracted item gets: the block's answer and a link back to the note.
pub fn answer_with_source(block: &InlineBlock, note_title: &str) -> String {
    let target = source_target(note_title, block.heading.as_deref());
    format!("{}\n\nSource: [[{target}]]", block.back)
}

/// A card's back or a question's reference without the trailing source line, for comparing answers.
pub fn without_source(text: &str) -> &str {
    match text.trim_end().rsplit_once("\n\nSource: [[") {
        Some((head, tail)) if tail.ends_with("]]") && !tail.contains('\n') => head,
        _ => text,
    }
}

/// The items of `folder` extracted from the note with the id `from`: (key, uid, path, answer).
fn extracted_items(
    session: &VaultSession,
    folder: &str,
    from: &str,
) -> Vec<(String, Uid, String, String)> {
    let Ok(dir) = fs::read_dir(session.root.join(folder)) else {
        return Vec::new();
    };
    dir.filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .filter_map(|e| {
            let text = fs::read_to_string(e.path()).ok()?;
            let doc = parse_doc(&text).ok()?;
            let answer = match &doc {
                Doc::Card(c) => c.back.clone(),
                Doc::Question(q) => q.reference.clone(),
                _ => return None,
            };
            (extra_value(&doc, FROM_FIELD)? == from).then_some(())?;
            let key = extra_value(&doc, KEY_FIELD)?;
            let name = e.file_name().to_string_lossy().into_owned();
            Some((key, doc_meta(&doc).uid, format!("{folder}/{name}"), answer))
        })
        .collect()
}

fn entry_of(block: InlineBlock, title: &str, have: &[(String, Uid, String, String)]) -> SyncEntry {
    let answer = answer_with_source(&block, title);
    let key = extract_key(&block);
    let found = have.iter().find(|(k, ..)| *k == key);
    let change = match found {
        None => SyncChange::New,
        Some((.., old)) if old.trim_end() == answer.trim_end() => SyncChange::Unchanged,
        Some(_) => SyncChange::Updated,
    };
    SyncEntry {
        block,
        change,
        existing: found.map(|(_, uid, path, _)| (*uid, path.clone())),
        answer,
    }
}

/// What syncing the note `uid` would do. `body` is the text being edited, when it differs from the file.
pub fn plan_inline_sync(
    session: &VaultSession,
    uid: Uid,
    body: Option<&str>,
) -> Result<SyncPlan, ServiceError> {
    let loaded = load_document(session, uid)?;
    let Doc::Note(note) = &loaded.doc else {
        return Err(ServiceError::Invalid(
            "only a note has inline cards".to_owned(),
        ));
    };
    let folder = loaded.path.rsplit_once('/').map_or("", |(f, _)| f);
    if folder.is_empty() {
        return Err(ServiceError::Invalid(
            "the note is not in a stack, so its cards have nowhere to go".to_owned(),
        ));
    }
    let have = extracted_items(session, folder, &format_ulid(uid));
    let title = note.meta.title.clone();
    let mut seen = std::collections::HashSet::new();
    let entries = extract_inline(body.unwrap_or(&note.body))
        .into_iter()
        .filter(|b| seen.insert(extract_key(b)))
        .map(|b| entry_of(b, &title, &have))
        .collect();
    Ok(SyncPlan {
        note: uid,
        title,
        folder: folder.to_owned(),
        tags: note.meta.tags.clone(),
        entries,
    })
}

/// The kind of item a block becomes.
pub fn kind_of(block: &InlineBlock) -> studyfrog_core::plan::ItemKind {
    match block.kind {
        InlineKind::Card => studyfrog_core::plan::ItemKind::Card,
        InlineKind::Question => studyfrog_core::plan::ItemKind::Question,
    }
}

pub use crate::inline_sync_apply::{SyncOutcome, apply_inline_sync};

/// Milliseconds for an `updated` stamp.
pub(crate) fn stamp(now_ms: u64) -> UnixMs {
    UnixMs(i64::try_from(now_ms).unwrap_or(i64::MAX))
}
