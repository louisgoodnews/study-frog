//! The contrastive drill (STORY-0111, `docs/specs/contrastive-drill.md`): 2 to 4 items that get confused,
//! read side by side, and the distinction the learner writes, kept as a hint on each item or as a
//! comparison note linked to all of them.

use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{Doc, doc_meta};

use crate::content::load_item_content;
use crate::session::{ServiceError, VaultSession};

/// Fewest and most items a drill compares.
pub const CONTRAST_SIZE: (usize, usize) = (2, 4);

/// One item of a drill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContrastItem {
    /// The item.
    pub uid: Uid,
    /// Its title.
    pub title: String,
    /// What it asks.
    pub prompt: String,
    /// What it answers.
    pub answer: String,
}

/// Where the distinction goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContrastTarget {
    /// A `> Distinction` hint at the end of each item's answer (replaced on a later save).
    #[default]
    Hints,
    /// A comparison note linking every item (updated on a later save of the same items).
    Note,
}

/// The prefix of a distinction hint.
pub const HINT: &str = "> Distinction";

/// Check the number of items.
pub fn check_size(n: usize) -> Result<(), ServiceError> {
    let (lo, hi) = CONTRAST_SIZE;
    if (lo..=hi).contains(&n) {
        Ok(())
    } else {
        Err(ServiceError::Invalid(format!(
            "a contrastive drill compares {lo} to {hi} items, not {n}"
        )))
    }
}

/// The items of a drill, in the given order.
pub fn load_contrast(
    session: &VaultSession,
    uids: &[Uid],
) -> Result<Vec<ContrastItem>, ServiceError> {
    check_size(uids.len())?;
    uids.iter()
        .map(|uid| {
            let c = load_item_content(session, *uid)?;
            Ok(ContrastItem {
                uid: *uid,
                title: c.title,
                prompt: c.prompt,
                answer: c.answer,
            })
        })
        .collect()
}

/// The text with its distinction hint (the trailing `> Distinction` lines) set to `hint`.
pub fn with_hint(text: &str, hint: &str) -> String {
    let kept: Vec<&str> = text.lines().collect();
    let cut = kept
        .iter()
        .position(|l| l.starts_with(HINT))
        .unwrap_or(kept.len());
    let head = kept[..cut].join("\n");
    format!("{}\n\n{hint}", head.trim_end())
}

/// The hint written on an item: the distinction and links to the other items.
pub fn hint_line(others: &[String], text: &str) -> String {
    let links: Vec<String> = others.iter().map(|t| format!("[[{t}]]")).collect();
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    format!("{HINT} (vs {}): {one_line}", links.join(", "))
}

/// An answer without what StudyFrog added to it (a distinction hint, a source line): what a typed answer
/// is compared with.
pub fn answer_core(text: &str) -> &str {
    let cut = text.find(&format!("\n{HINT}")).map_or(text, |i| &text[..i]);
    crate::inline_sync::without_source(cut.trim_end())
}

/// The text the item's hint goes into.
pub(crate) fn answer_mut(doc: &mut Doc) -> Option<&mut String> {
    match doc {
        Doc::Card(c) => Some(&mut c.back),
        Doc::Question(q) => Some(&mut q.reference),
        Doc::Note(n) => Some(&mut n.body),
        _ => None,
    }
}

/// The titles of the documents, for links.
pub(crate) fn titles(docs: &[Doc]) -> Vec<String> {
    docs.iter().map(|d| doc_meta(d).title.clone()).collect()
}

pub use crate::contrast_save::{ContrastSaved, save_contrast};
