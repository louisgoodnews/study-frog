//! Pure document layout engine turning parsed Markdown blocks into paginated pages.

use studyfrog_md::blocks::{Block, Link, Span, plain_text};

use super::geometry::{
    PageGeometry, PageSize, content_box, default_geometry, page_size_dimensions,
};
use super::layout_blocks::{layout_heading, layout_image, layout_paragraph, layout_rule};
use super::layout_code::layout_code_block;
use super::layout_lists::{layout_list_item, layout_quote};
use super::layout_table::layout_table_block;
use super::layout_toc::layout_toc_block;
use super::primitives::{HeadingOutline, LayoutPage, PagedDocument, new_layout_page};

/// Configuration options for document layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutOptions {
    /// Page geometry (margins, paper size).
    pub geometry: PageGeometry,
    /// Document title for running headers and metadata.
    pub title: Option<String>,
    /// Document author.
    pub author: Option<String>,
    /// Vault name or sub-label for running footer.
    pub vault_name: Option<String>,
    /// Whether to print page numbers in running footers.
    pub page_numbers: bool,
    /// Base font size in points for body text.
    pub font_size: f32,
}

/// Default layout options for A4 paper.
pub fn default_layout_options() -> LayoutOptions {
    LayoutOptions {
        geometry: default_geometry(PageSize::A4),
        title: None,
        author: Some("StudyFrog".to_string()),
        vault_name: None,
        page_numbers: true,
        font_size: 10.0,
    }
}

fn scan_headings(blocks: &[Block]) -> Vec<HeadingOutline> {
    let mut headings = Vec::new();
    for b in blocks {
        if let Block::Heading { level, spans } = b {
            headings.push(HeadingOutline {
                title: plain_text(spans),
                level: *level,
                page_index: 0,
                y: 0.0,
            });
        }
    }
    headings
}

#[allow(clippy::too_many_arguments)]
fn layout_block_list(
    blocks: &[Block],
    toc_headings: &[HeadingOutline],
    font_size: f32,
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    outlines: &mut Vec<HeadingOutline>,
    box_coords: (f32, f32, f32, f32),
) {
    for block in blocks {
        match block {
            Block::Heading { level, spans } => {
                layout_heading(*level, spans, page, pages, cursor_y, outlines, box_coords);
            }
            Block::Paragraph(spans) => {
                if let [
                    Span {
                        text,
                        link: Some(Link::Image(path)),
                        ..
                    },
                ] = spans.as_slice()
                {
                    layout_image(text, path, page, pages, cursor_y, box_coords);
                    continue;
                }
                layout_paragraph(spans, font_size, page, pages, cursor_y, box_coords);
            }
            Block::Code { lang, text, tokens } => {
                layout_code_block(lang, text, tokens, page, pages, cursor_y, box_coords);
            }
            Block::Quote(inner) => {
                layout_quote(
                    inner,
                    page,
                    pages,
                    cursor_y,
                    &mut |sub, p, ps, cy, outs, bounds| {
                        layout_block_list(sub, toc_headings, font_size, p, ps, cy, outs, bounds);
                    },
                    outlines,
                    box_coords,
                );
            }
            Block::List {
                ordered,
                start,
                items,
            } => {
                let mut num = *start;
                for item in items {
                    let marker = item.marker.clone().unwrap_or_else(|| {
                        if *ordered {
                            let m = format!("{num}.");
                            num += 1;
                            m
                        } else {
                            "•".to_string()
                        }
                    });
                    layout_list_item(
                        &marker,
                        item.checked,
                        item,
                        0,
                        page,
                        pages,
                        cursor_y,
                        &mut |sub, p, ps, cy, outs, bounds| {
                            layout_block_list(
                                sub,
                                toc_headings,
                                font_size,
                                p,
                                ps,
                                cy,
                                outs,
                                bounds,
                            );
                        },
                        outlines,
                        box_coords,
                    );
                }
            }
            Block::Rule => {
                layout_rule(page, pages, cursor_y, box_coords);
            }
            Block::Table {
                aligns,
                header,
                rows,
            } => {
                layout_table_block(aligns, header, rows, page, pages, cursor_y, box_coords);
            }
            Block::Toc => {
                layout_toc_block(toc_headings, page, pages, cursor_y, box_coords);
            }
        }
    }
}

/// Pure function turning parsed Markdown blocks into a [`PagedDocument`] with no I/O.
pub fn layout_document(blocks: &[Block], options: &LayoutOptions) -> PagedDocument {
    let (w, h) = page_size_dimensions(options.geometry.size);
    let box_coords = content_box(&options.geometry);
    let content_top = box_coords.1;

    let has_toc = blocks.iter().any(|b| matches!(b, Block::Toc));
    let preliminary_headings = if has_toc {
        let mut p = new_layout_page(0, w, h);
        let mut ps = Vec::new();
        let mut cy = content_top;
        let mut dummy_outlines = Vec::new();
        let empty_toc = scan_headings(blocks);
        layout_block_list(
            blocks,
            &empty_toc,
            options.font_size,
            &mut p,
            &mut ps,
            &mut cy,
            &mut dummy_outlines,
            box_coords,
        );
        dummy_outlines
    } else {
        Vec::new()
    };

    let mut current_page = new_layout_page(0, w, h);
    let mut finished_pages = Vec::new();
    let mut cursor_y = content_top;
    let mut outlines = Vec::new();

    layout_block_list(
        blocks,
        &preliminary_headings,
        options.font_size,
        &mut current_page,
        &mut finished_pages,
        &mut cursor_y,
        &mut outlines,
        box_coords,
    );

    finished_pages.push(current_page);

    // Renumber pages
    for (i, p) in finished_pages.iter_mut().enumerate() {
        p.page_number = i;
    }

    let doc_title = options.title.clone().or_else(|| {
        outlines
            .iter()
            .find(|h| h.level == 1)
            .map(|h| h.title.clone())
    });

    PagedDocument {
        pages: finished_pages,
        headings: outlines,
        title: doc_title,
    }
}
