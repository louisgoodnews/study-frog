//! Parsed documents: one struct per kind, plus verbatim unknown keys.
//! Stack membership comes from the folder, so docs carry no stack reference.

use std::collections::BTreeMap;

use studyfrog_core::civil::Date;
use studyfrog_core::diary::RatingValue;
use studyfrog_core::meta::Meta;
use studyfrog_core::uid::Uid;

/// An unknown top-level key kept verbatim so a rewrite never loses user data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEntry {
    /// Key name.
    pub key: String,
    /// Verbatim text of the key and its continuation lines.
    pub text: String,
}

/// `_stack.md`: the stack's identity and description. Its settings live in `_stack.toml`.
#[derive(Debug, Clone, PartialEq)]
pub struct StackDoc {
    /// Common fields.
    pub meta: Meta,
    /// The `description` key; empty when absent.
    pub description: String,
    /// Markdown body.
    pub body: String,
    /// Unknown keys.
    pub extra: Vec<RawEntry>,
}

/// A note file.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteDoc {
    /// Common fields.
    pub meta: Meta,
    /// `rehearse_title`.
    pub rehearse_title: bool,
    /// Markdown body.
    pub body: String,
    /// Unknown keys.
    pub extra: Vec<RawEntry>,
}

/// A card file.
#[derive(Debug, Clone, PartialEq)]
pub struct CardDoc {
    /// Common fields.
    pub meta: Meta,
    /// `reversible`.
    pub reversible: bool,
    /// Text before the `???` line.
    pub front: String,
    /// Text after the `???` line.
    pub back: String,
    /// Unknown keys.
    pub extra: Vec<RawEntry>,
}

/// A question file.
#[derive(Debug, Clone, PartialEq)]
pub struct QuestionDoc {
    /// Common fields.
    pub meta: Meta,
    /// Text before the `???` line.
    pub prompt: String,
    /// Text after the `???` line.
    pub reference: String,
    /// `accepted`.
    pub accepted: Vec<String>,
    /// `rubric`.
    pub rubric: Vec<String>,
    /// `min_score`, resolved to the default when absent.
    pub min_score: f32,
    /// Unknown keys.
    pub extra: Vec<RawEntry>,
}

/// A diary entry file.
#[derive(Debug, Clone, PartialEq)]
pub struct DiaryDoc {
    /// Common fields.
    pub meta: Meta,
    /// Civil date.
    pub date: Date,
    /// Explicit title from front-matter; `None` when defaulted.
    pub custom_title: Option<String>,
    /// Study time in minutes.
    pub study_minutes: Option<u32>,
    /// Self-assessment ratings.
    pub ratings: BTreeMap<String, RatingValue>,
    /// Linked rehearsal runs.
    pub runs: Vec<Uid>,
    /// Linked stacks.
    pub stacks: Vec<String>,
    /// Linked items.
    pub items: Vec<Uid>,
    /// Linked exams.
    pub exams: Vec<String>,
    /// Reflections Markdown body.
    pub body: String,
    /// Unknown keys.
    pub extra: Vec<RawEntry>,
}

/// Any parsed file.
#[derive(Debug, Clone, PartialEq)]
pub enum Doc {
    /// A stack definition.
    Stack(StackDoc),
    /// A note.
    Note(NoteDoc),
    /// A card.
    Card(CardDoc),
    /// A question.
    Question(QuestionDoc),
    /// A diary entry.
    Diary(DiaryDoc),
}

/// The `kind` front-matter value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `stack`
    Stack,
    /// `note`
    Note,
    /// `card`
    Card,
    /// `question`
    Question,
    /// `diary`
    Diary,
}

/// Name as written in front-matter.
pub fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Stack => "stack",
        Kind::Note => "note",
        Kind::Card => "card",
        Kind::Question => "question",
        Kind::Diary => "diary",
    }
}

/// Inverse of [`kind_name`].
pub fn kind_from_name(name: &str) -> Option<Kind> {
    [
        Kind::Stack,
        Kind::Note,
        Kind::Card,
        Kind::Question,
        Kind::Diary,
    ]
    .into_iter()
    .find(|k| kind_name(*k) == name)
}

/// The kind of a parsed document.
pub fn doc_kind(doc: &Doc) -> Kind {
    match doc {
        Doc::Stack(_) => Kind::Stack,
        Doc::Note(_) => Kind::Note,
        Doc::Card(_) => Kind::Card,
        Doc::Question(_) => Kind::Question,
        Doc::Diary(_) => Kind::Diary,
    }
}

const COMMON: [&str; 7] = [
    "id", "kind", "title", "tags", "aliases", "created", "updated",
];

const DIARY_OWN: [&str; 19] = [
    "date",
    "study_minutes",
    "runs",
    "stacks",
    "items",
    "exams",
    "performance",
    "success",
    "confidence",
    "effort",
    "focus",
    "energy",
    "difficulty",
    "satisfaction",
    "understanding",
    "mood",
    "stress",
    "motivation",
    "sleep",
];

/// Keys this kind understands; every other top-level key is preserved as an unknown entry.
pub fn known_keys(kind: Kind) -> Vec<&'static str> {
    let own: &[&str] = match kind {
        Kind::Stack => &["description"],
        Kind::Note => &["rehearse_title"],
        Kind::Card => &["reversible"],
        Kind::Question => &["accepted", "rubric", "min_score"],
        Kind::Diary => &DIARY_OWN,
    };
    COMMON.iter().chain(own).copied().collect()
}

/// The common fields of any document.
pub fn doc_meta(doc: &Doc) -> &Meta {
    match doc {
        Doc::Stack(d) => &d.meta,
        Doc::Note(d) => &d.meta,
        Doc::Card(d) => &d.meta,
        Doc::Question(d) => &d.meta,
        Doc::Diary(d) => &d.meta,
    }
}

/// The Markdown texts of a document, for link extraction and search.
pub fn doc_texts(doc: &Doc) -> Vec<&str> {
    match doc {
        Doc::Stack(d) => vec![&d.body],
        Doc::Note(d) => vec![&d.body],
        Doc::Card(d) => vec![&d.front, &d.back],
        Doc::Question(d) => vec![&d.prompt, &d.reference],
        Doc::Diary(d) => vec![&d.body],
    }
}

pub use crate::build_diary::{diary_doc_to_entry, diary_entry_to_doc};
