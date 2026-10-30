//! Markdown to a small block-and-span model for previews. Pure. CommonMark comes from
//! `pulldown-cmark` (a hard general problem, P10); wikilinks are added here.

/// Inline formatting of a span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    /// Strong emphasis.
    pub bold: bool,
    /// Emphasis.
    pub italic: bool,
    /// Strikethrough.
    pub strike: bool,
    /// Inline code.
    pub code: bool,
}

/// Where a link points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    /// `[[Target]]`: a note title or alias.
    Wiki(String),
    /// An ordinary URL.
    Url(String),
}

/// A run of text with one style and at most one link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Visible text.
    pub text: String,
    /// Formatting.
    pub style: Style,
    /// Link, if the text is one.
    pub link: Option<Link>,
}

/// A list item (task items carry their checkbox).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListItem {
    /// `Some(done)` for task items.
    pub checked: Option<bool>,
    /// The item's content.
    pub blocks: Vec<Block>,
}

/// A block of the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A heading, level 1 to 6.
    Heading {
        /// Level.
        level: u8,
        /// Content.
        spans: Vec<Span>,
    },
    /// A paragraph.
    Paragraph(Vec<Span>),
    /// A code block.
    Code {
        /// Info string (language), possibly empty.
        lang: String,
        /// The code.
        text: String,
    },
    /// A block quote.
    Quote(Vec<Block>),
    /// A list.
    List {
        /// Numbered rather than bulleted.
        ordered: bool,
        /// First number of a numbered list.
        start: u64,
        /// Items.
        items: Vec<ListItem>,
    },
    /// A thematic break.
    Rule,
}

/// Parse Markdown into blocks. Never fails; unsupported constructs degrade to plain text.
pub fn parse_blocks(markdown: &str) -> Vec<Block> {
    crate::blocks_build::build(markdown)
}

/// The visible text of a span list, for tests and accessibility labels.
pub fn plain_text(spans: &[Span]) -> String {
    spans.iter().map(|s| s.text.as_str()).collect()
}
