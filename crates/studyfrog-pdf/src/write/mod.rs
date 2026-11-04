//! Markdown layout engine and PDF document writer.
//!
//! Provides a pure layout pass turning parsed Markdown blocks into pages and
//! a writer producing valid PDF documents with bookmarks, links, and selectable text.

pub mod geometry;
pub mod layout;
pub mod layout_blocks;
pub mod layout_code;
pub mod layout_lists;
pub mod layout_table;
pub mod layout_toc;
pub mod metrics;
pub mod palette;
pub mod pdf;
pub mod pdf_fonts;
pub mod pdf_outline;
pub mod pdf_stream;
pub mod primitives;
pub mod to_unicode;
pub mod wrap;

pub use geometry::{Margins, PageGeometry, PageSize, default_geometry, default_margins};
pub use layout::{LayoutOptions, default_layout_options, layout_document};
pub use pdf::write_pdf;
pub use primitives::{
    HeadingOutline, LayoutCodeBlock, LayoutHeading, LayoutImage, LayoutItem, LayoutPage,
    LayoutQuoteBar, LayoutRule, LayoutTable, LayoutTableCell, LayoutTaskCheckbox, PagedDocument,
    StyledRun, TextLine,
};

use crate::error::PdfError;
use studyfrog_md::blocks::Block;

/// Export parsed Markdown blocks directly to valid PDF bytes.
pub fn export_markdown_to_pdf(
    blocks: &[Block],
    options: &LayoutOptions,
) -> Result<Vec<u8>, PdfError> {
    let paged = layout_document(blocks, options);
    write_pdf(&paged, options)
}
