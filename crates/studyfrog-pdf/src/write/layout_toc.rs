//! Table of contents layout with dotted leaders and target page numbers.

use studyfrog_md::blocks::Link;

use super::layout_blocks::ensure_space;
use super::metrics::measure_str;
use super::palette::{COLOR_LINK, COLOR_MUTED, COLOR_TEXT_BODY};
use super::primitives::{HeadingOutline, LayoutItem, LayoutPage, StyledRun, TextLine};

const TOC_LINE_HEIGHT: f32 = 16.0;
const TOC_FONT_SIZE: f32 = 9.5;

/// Layout table of contents entries from document headings.
pub fn layout_toc_block(
    headings: &[HeadingOutline],
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    (content_x, content_top, content_w, content_h): (f32, f32, f32, f32),
) {
    if headings.is_empty() {
        ensure_space(
            TOC_LINE_HEIGHT,
            page,
            pages,
            cursor_y,
            content_top,
            content_h,
        );
        page.items.push(LayoutItem::Text(TextLine {
            x: content_x,
            y: *cursor_y,
            runs: vec![StyledRun {
                text: "This document has no headings yet.".to_string(),
                bold: false,
                italic: true,
                underline: false,
                strike: false,
                code: false,
                color: COLOR_MUTED,
                font_size: 10.0,
                width: measure_str("This document has no headings yet.", 10.0, false, false),
                link: None,
            }],
            height: TOC_LINE_HEIGHT,
            baseline: 12.0,
        }));
        *cursor_y += TOC_LINE_HEIGHT + 8.0;
        return;
    }

    for h in headings {
        ensure_space(
            TOC_LINE_HEIGHT,
            page,
            pages,
            cursor_y,
            content_top,
            content_h,
        );

        let indent = ((h.level.saturating_sub(1)) as f32 * 14.0).min(content_w * 0.4);
        let title_x = content_x + indent;
        let title_w = measure_str(&h.title, TOC_FONT_SIZE, false, false);
        let page_str = format!("{}", h.page_index + 1);
        let page_w = measure_str(&page_str, TOC_FONT_SIZE, true, false);

        let leader_avail = (content_w - indent - title_w - page_w - 12.0).max(0.0);
        let dot_w = measure_str(". ", TOC_FONT_SIZE, false, false);
        let dot_count = if dot_w > 0.0 {
            (leader_avail / dot_w).floor() as usize
        } else {
            0
        };
        let dots = ". ".repeat(dot_count);
        let actual_dots_w = measure_str(&dots, TOC_FONT_SIZE, false, false);

        let mut runs = Vec::new();
        // Title run (clickable with heading target)
        runs.push(StyledRun {
            text: h.title.clone(),
            bold: h.level <= 2,
            italic: false,
            underline: false,
            strike: false,
            code: false,
            color: COLOR_LINK,
            font_size: TOC_FONT_SIZE,
            width: title_w,
            link: Some(Link::Wiki(h.title.clone())),
        });

        if dot_count > 0 {
            runs.push(StyledRun {
                text: dots,
                bold: false,
                italic: false,
                underline: false,
                strike: false,
                code: false,
                color: COLOR_MUTED,
                font_size: TOC_FONT_SIZE,
                width: actual_dots_w,
                link: None,
            });
        }

        runs.push(StyledRun {
            text: page_str,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            code: false,
            color: COLOR_TEXT_BODY,
            font_size: TOC_FONT_SIZE,
            width: page_w,
            link: Some(Link::Wiki(h.title.clone())),
        });

        page.items.push(LayoutItem::Text(TextLine {
            x: title_x,
            y: *cursor_y,
            runs,
            height: TOC_LINE_HEIGHT,
            baseline: 12.0,
        }));
        *cursor_y += TOC_LINE_HEIGHT;
    }
    *cursor_y += 8.0;
}
