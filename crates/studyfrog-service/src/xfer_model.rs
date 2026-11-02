//! What export writes and import reads: a bundle of stacks, items and files, whatever the file format (`docs/specs/interchange.md`).

use studyfrog_core::plan::ItemKind;
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;

/// A stack.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XStack {
    /// Its folder, `/`-separated.
    pub folder: String,
    /// Its title.
    pub title: String,
    /// Its description.
    pub description: String,
}

/// A review of an item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XReview {
    /// When, in Unix milliseconds.
    pub at_ms: i64,
    /// The rating.
    pub rating: Rating,
    /// Days since the review before.
    pub elapsed_days: f32,
}

/// A note, card or question.
#[derive(Debug, Clone, PartialEq)]
pub struct XItem {
    /// Its id, when the source has one.
    pub id: Option<Uid>,
    /// What it is.
    pub kind: ItemKind,
    /// Its title (empty: made from the text).
    pub title: String,
    /// A note's body, a card's front, a question's prompt.
    pub text: String,
    /// A card's back or a question's reference answer.
    pub answer: String,
    /// Its tags.
    pub tags: Vec<String>,
    /// The folder of its stack.
    pub stack: String,
    /// The status word it states.
    pub status: Option<String>,
    /// The priority word it states.
    pub priority: Option<String>,
    /// The difficulty word it states.
    pub difficulty: Option<String>,
    /// Its reviews, when scheduling is included.
    pub reviews: Vec<XReview>,
}

/// Everything in a file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Bundle {
    /// The stacks.
    pub stacks: Vec<XStack>,
    /// The items.
    pub items: Vec<XItem>,
    /// Pictures and PDFs: a vault-relative path and the bytes.
    pub files: Vec<(String, Vec<u8>)>,
    /// Whether the reviews are included.
    pub scheduling: bool,
    /// Things the reader could not use or kept as text, for the preview.
    pub warnings: Vec<String>,
}

/// A new item with nothing but its kind, text and stack.
pub fn item(kind: ItemKind, stack: &str, text: &str, answer: &str) -> XItem {
    XItem {
        id: None,
        kind,
        title: String::new(),
        text: text.to_owned(),
        answer: answer.to_owned(),
        tags: Vec::new(),
        stack: stack.to_owned(),
        status: None,
        priority: None,
        difficulty: None,
        reviews: Vec::new(),
    }
}

/// Whether a stack folder is safe to create inside a vault: relative, no `..`, no empty or hidden parts.
pub fn safe_folder(folder: &str) -> bool {
    !folder.is_empty()
        && !folder.starts_with('/')
        && !folder.contains('\\')
        && folder.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.starts_with('.')
                && !p.chars().any(char::is_control)
        })
}
