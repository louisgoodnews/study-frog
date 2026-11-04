//! Layout for lists (bullet, ordered, tasks) and blockquotes.

use studyfrog_md::blocks::{Block, ListItem};

use super::layout_blocks::ensure_space;
use super::palette::{COLOR_QUOTE_BAR, COLOR_TEXT_BODY};
use super::primitives::{
    HeadingOutline, LayoutItem, LayoutPage, LayoutQuoteBar, LayoutTaskCheckbox, StyledRun, TextLine,
};

type SubLayoutFn<'a> = &'a mut dyn FnMut(
    &[Block],
    &mut LayoutPage,
    &mut Vec<LayoutPage>,
    &mut f32,
    &mut Vec<HeadingOutline>,
    (f32, f32, f32, f32),
);

/// Layout a list item with its marker (bullet, number, or checkbox) and nested content.
#[allow(clippy::too_many_arguments)]
pub fn layout_list_item(
    marker: &str,
    checked: Option<bool>,
    item: &ListItem,
    indent_level: usize,
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    layout_sub_blocks: SubLayoutFn<'_>,
    outlines: &mut Vec<HeadingOutline>,
    (content_x, content_top, content_w, content_h): (f32, f32, f32, f32),
) {
    let indent = 18.0 * (indent_level as f32 + 1.0);
    let item_x = content_x + indent;
    let item_w = (content_w - indent).max(20.0);

    ensure_space(14.0, page, pages, cursor_y, content_top, content_h);
    let marker_y = *cursor_y;

    if let Some(is_checked) = checked {
        page.items
            .push(LayoutItem::TaskCheckbox(LayoutTaskCheckbox {
                x: item_x - 14.0,
                y: marker_y + 2.0,
                size: 8.5,
                checked: is_checked,
            }));
    } else {
        page.items.push(LayoutItem::Text(TextLine {
            x: item_x - 14.0,
            y: marker_y,
            runs: vec![StyledRun {
                text: marker.to_string(),
                bold: false,
                italic: false,
                underline: false,
                strike: false,
                code: false,
                color: COLOR_TEXT_BODY,
                font_size: 10.0,
                width: 10.0,
                link: None,
            }],
            height: 14.0,
            baseline: 10.0,
        }));
    }

    layout_sub_blocks(
        &item.blocks,
        page,
        pages,
        cursor_y,
        outlines,
        (item_x, content_top, item_w, content_h),
    );
}

/// Layout a blockquote with a vertical bar and italicized inner content.
#[allow(clippy::too_many_arguments)]
pub fn layout_quote(
    blocks: &[Block],
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    layout_sub_blocks: SubLayoutFn<'_>,
    outlines: &mut Vec<HeadingOutline>,
    (content_x, content_top, content_w, content_h): (f32, f32, f32, f32),
) {
    let quote_indent = 16.0;
    let quote_x = content_x + quote_indent;
    let quote_w = (content_w - quote_indent).max(20.0);

    let start_y = *cursor_y;
    let start_page = pages.len();

    let mut italic_blocks = blocks.to_vec();
    for b in &mut italic_blocks {
        if let Block::Paragraph(spans) = b {
            for s in spans {
                s.style.italic = true;
            }
        }
    }

    layout_sub_blocks(
        &italic_blocks,
        page,
        pages,
        cursor_y,
        outlines,
        (quote_x, content_top, quote_w, content_h),
    );

    let end_y = *cursor_y;
    let end_page = pages.len();

    if start_page == end_page {
        page.items.push(LayoutItem::QuoteBar(LayoutQuoteBar {
            x: content_x + 4.0,
            y: start_y,
            width: 2.0,
            height: (end_y - start_y).max(14.0),
            color: COLOR_QUOTE_BAR,
        }));
    } else {
        page.items.push(LayoutItem::QuoteBar(LayoutQuoteBar {
            x: content_x + 4.0,
            y: content_top,
            width: 2.0,
            height: (end_y - content_top).max(14.0),
            color: COLOR_QUOTE_BAR,
        }));
    }
}
