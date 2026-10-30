//! Parsed documents: one struct per kind, plus verbatim unknown keys.
//! Stack membership comes from the folder, so docs carry no stack reference.

use studyfrog_core::meta::Meta;

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
}

/// Name as written in front-matter.
pub fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Stack => "stack",
        Kind::Note => "note",
        Kind::Card => "card",
        Kind::Question => "question",
    }
}

/// Inverse of [`kind_name`].
pub fn kind_from_name(name: &str) -> Option<Kind> {
    [Kind::Stack, Kind::Note, Kind::Card, Kind::Question]
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
    }
}

const COMMON: [&str; 7] = [
    "id", "kind", "title", "tags", "aliases", "created", "updated",
];

/// Keys this kind understands; every other top-level key is preserved as an unknown entry.
pub fn known_keys(kind: Kind) -> Vec<&'static str> {
    let own: &[&str] = match kind {
        Kind::Stack => &["description"],
        Kind::Note => &["rehearse_title"],
        Kind::Card => &["reversible"],
        Kind::Question => &["accepted", "rubric", "min_score"],
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
    }
}

/// The Markdown texts of a document, for link extraction and search.
pub fn doc_texts(doc: &Doc) -> Vec<&str> {
    match doc {
        Doc::Stack(d) => vec![&d.body],
        Doc::Note(d) => vec![&d.body],
        Doc::Card(d) => vec![&d.front, &d.back],
        Doc::Question(d) => vec![&d.prompt, &d.reference],
    }
}
