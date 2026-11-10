//! Domain structures for timestamped comments on vault items (STORY-0135).

/// A timestamped comment attached to an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemComment {
    /// Unique identifier for the comment.
    pub id: String,
    /// Author label ("user", "agent", or custom name).
    pub author: String,
    /// Created timestamp in Unix milliseconds.
    pub created: i64,
    /// Markdown body text of the comment.
    pub text: String,
}

/// Helper to create a new comment with current timestamp.
pub fn new_comment(
    id: impl Into<String>,
    author: impl Into<String>,
    created: i64,
    text: impl Into<String>,
) -> ItemComment {
    ItemComment {
        id: id.into(),
        author: author.into(),
        created,
        text: text.into(),
    }
}
