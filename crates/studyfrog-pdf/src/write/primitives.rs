//! Layout primitives and data structures produced by the pure layout engine.

use studyfrog_md::blocks::Link;

use super::palette::Rgb;

/// A styled run of text with uniform formatting, font, and link.
#[derive(Debug, Clone, PartialEq)]
pub struct StyledRun {
    /// Text content of this run.
    pub text: String,
    /// Strong emphasis.
    pub bold: bool,
    /// Emphasis.
    pub italic: bool,
    /// Underline.
    pub underline: bool,
    /// Strikethrough.
    pub strike: bool,
    /// Monospace inline code.
    pub code: bool,
    /// Text color.
    pub color: Rgb,
    /// Font size in points.
    pub font_size: f32,
    /// Measured width in points.
    pub width: f32,
    /// Target link if this run is clickable.
    pub link: Option<Link>,
}

/// A horizontal line of styled text runs at a specific `(x, y)` on the page.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    /// Horizontal position of the line from page left.
    pub x: f32,
    /// Vertical position of the line top from page top.
    pub y: f32,
    /// Runs making up this line.
    pub runs: Vec<StyledRun>,
    /// Line height in points.
    pub height: f32,
    /// Baseline offset from top of line in points.
    pub baseline: f32,
}

/// A laid-out heading.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutHeading {
    /// Heading level (1..=6).
    pub level: u8,
    /// Plain title string for bookmarks/outlines.
    pub title: String,
    /// The formatted text line.
    pub line: TextLine,
}

/// A horizontal dividing rule.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutRule {
    /// Left position.
    pub x: f32,
    /// Top position.
    pub y: f32,
    /// Width in points.
    pub width: f32,
    /// Stroke thickness in points.
    pub thickness: f32,
}

/// Left vertical decorative quote bar.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutQuoteBar {
    /// Left position.
    pub x: f32,
    /// Top position.
    pub y: f32,
    /// Bar width.
    pub width: f32,
    /// Bar height.
    pub height: f32,
    /// Stroke/fill color.
    pub color: Rgb,
}

/// A laid-out code block container with lines.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutCodeBlock {
    /// Left position.
    pub x: f32,
    /// Top position.
    pub y: f32,
    /// Width in points.
    pub width: f32,
    /// Height in points.
    pub height: f32,
    /// Code lines inside the box.
    pub lines: Vec<TextLine>,
    /// Language info string.
    pub lang: String,
    /// Whether this block continues from a previous page.
    pub continued: bool,
}

/// A single cell in a laid-out table.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutTableCell {
    /// Left position.
    pub x: f32,
    /// Top position.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
    /// Lines inside this cell.
    pub lines: Vec<TextLine>,
    /// Whether this cell is part of the table header.
    pub is_header: bool,
    /// Cell background fill color.
    pub bg_color: Option<Rgb>,
}

/// A laid-out table grid.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutTable {
    /// Left position.
    pub x: f32,
    /// Top position.
    pub y: f32,
    /// Total width.
    pub width: f32,
    /// Total height.
    pub height: f32,
    /// Cells in the table.
    pub cells: Vec<LayoutTableCell>,
}

/// A square task list checkbox.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutTaskCheckbox {
    /// Left position.
    pub x: f32,
    /// Top position.
    pub y: f32,
    /// Size (width and height) of the box.
    pub size: f32,
    /// Whether checked.
    pub checked: bool,
}

/// Image placeholder box.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutImage {
    /// Left position.
    pub x: f32,
    /// Top position.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
    /// Alt text description.
    pub alt: String,
    /// Asset or vault relative path.
    pub path: String,
}

/// Individual item placed on a page.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutItem {
    /// Text line.
    Text(TextLine),
    /// Heading.
    Heading(LayoutHeading),
    /// Horizontal rule.
    Rule(LayoutRule),
    /// Quote vertical bar.
    QuoteBar(LayoutQuoteBar),
    /// Code block box and text.
    CodeBox(LayoutCodeBlock),
    /// Table grid and cells.
    Table(LayoutTable),
    /// Task list checkbox.
    TaskCheckbox(LayoutTaskCheckbox),
    /// Image placeholder box.
    Image(LayoutImage),
}

/// Single laid-out page containing positioned layout items.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutPage {
    /// 0-based page index.
    pub page_number: usize,
    /// Page width in points.
    pub width: f32,
    /// Page height in points.
    pub height: f32,
    /// Items placed on this page.
    pub items: Vec<LayoutItem>,
    /// Heading targets on this page: `(title, y)`.
    pub heading_destinations: Vec<(String, f32)>,
}

/// Outline entry for PDF document bookmarks.
#[derive(Debug, Clone, PartialEq)]
pub struct HeadingOutline {
    /// Plain title.
    pub title: String,
    /// Heading level (1..=6).
    pub level: u8,
    /// 0-based index of target page.
    pub page_index: usize,
    /// Vertical position from top of page in points.
    pub y: f32,
}

/// Full paginated document ready for serialization.
#[derive(Debug, Clone, PartialEq)]
pub struct PagedDocument {
    /// Pages in reading order.
    pub pages: Vec<LayoutPage>,
    /// Outline bookmarks collected from headings.
    pub headings: Vec<HeadingOutline>,
    /// Primary document title (if any).
    pub title: Option<String>,
}

/// Create an empty page for the given dimensions.
pub fn new_layout_page(page_number: usize, width: f32, height: f32) -> LayoutPage {
    LayoutPage {
        page_number,
        width,
        height,
        items: Vec::new(),
        heading_destinations: Vec::new(),
    }
}
