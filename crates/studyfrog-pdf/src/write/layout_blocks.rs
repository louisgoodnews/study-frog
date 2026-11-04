//! Layout for headings, paragraphs, lists, quotes, rules, and images.

use studyfrog_md::blocks::{Span, plain_text};

use super::primitives::{
    HeadingOutline, LayoutHeading, LayoutImage, LayoutItem, LayoutPage, LayoutRule,
};
use super::wrap::wrap_spans;

/// Advance to a new page if the given needed height exceeds the remaining space.
pub fn ensure_space(
    needed: f32,
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    content_top: f32,
    content_h: f32,
) {
    if *cursor_y + needed > content_h {
        pages.push(std::mem::replace(
            page,
            super::primitives::new_layout_page(pages.len() + 1, page.width, page.height),
        ));
        *cursor_y = content_top;
    }
}

/// Layout a heading with keep-with-next spacing.
#[allow(clippy::too_many_arguments)]
pub fn layout_heading(
    level: u8,
    spans: &[Span],
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    outlines: &mut Vec<HeadingOutline>,
    (content_x, content_top, content_w, content_h): (f32, f32, f32, f32),
) {
    let (font_size, line_h, space_before, space_after) = match level {
        1 => (24.0, 28.0, 18.0, 8.0),
        2 => (18.0, 22.0, 14.0, 6.0),
        3 => (14.0, 18.0, 10.0, 4.0),
        _ => (11.0, 14.0, 8.0, 3.0),
    };

    let mut bold_spans = spans.to_vec();
    for s in &mut bold_spans {
        s.style.bold = true;
    }
    let lines = wrap_spans(&bold_spans, content_w, font_size, line_h);
    let total_h = lines.len() as f32 * line_h;

    // Keep with next: ensure at least 40pt of following content fits
    if *cursor_y > content_top && (*cursor_y + space_before + total_h + 40.0 > content_h) {
        pages.push(std::mem::replace(
            page,
            super::primitives::new_layout_page(pages.len() + 1, page.width, page.height),
        ));
        *cursor_y = content_top;
    } else if *cursor_y > content_top {
        *cursor_y += space_before;
    }

    let title = plain_text(spans);
    let current_page_idx = pages.len();
    outlines.push(HeadingOutline {
        title: title.clone(),
        level,
        page_index: current_page_idx,
        y: *cursor_y,
    });
    page.heading_destinations.push((title.clone(), *cursor_y));

    for mut line in lines {
        ensure_space(line_h, page, pages, cursor_y, content_top, content_h);
        line.x = content_x;
        line.y = *cursor_y;
        page.items.push(LayoutItem::Heading(LayoutHeading {
            level,
            title: title.clone(),
            line,
        }));
        *cursor_y += line_h;
    }
    *cursor_y += space_after;
}

/// Layout a paragraph of styled spans.
pub fn layout_paragraph(
    spans: &[Span],
    font_size: f32,
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    (content_x, content_top, content_w, content_h): (f32, f32, f32, f32),
) {
    let line_h = (font_size * 1.4).max(8.0);
    let lines = wrap_spans(spans, content_w, font_size, line_h);
    for mut line in lines {
        ensure_space(line.height, page, pages, cursor_y, content_top, content_h);
        line.x = content_x;
        line.y = *cursor_y;
        page.items.push(LayoutItem::Text(line));
        *cursor_y += line_h;
    }
    *cursor_y += 8.0;
}

/// Layout a thematic horizontal rule.
pub fn layout_rule(
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    (content_x, content_top, content_w, content_h): (f32, f32, f32, f32),
) {
    *cursor_y += 12.0;
    ensure_space(1.0, page, pages, cursor_y, content_top, content_h);
    page.items.push(LayoutItem::Rule(LayoutRule {
        x: content_x,
        y: *cursor_y,
        width: content_w,
        thickness: 0.5,
    }));
    *cursor_y += 12.0;
}

/// Layout an image placeholder.
pub fn layout_image(
    alt: &str,
    path: &str,
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    (content_x, content_top, content_w, content_h): (f32, f32, f32, f32),
) {
    let box_h = 60.0;
    ensure_space(box_h + 8.0, page, pages, cursor_y, content_top, content_h);
    page.items.push(LayoutItem::Image(LayoutImage {
        x: content_x,
        y: *cursor_y,
        width: content_w,
        height: box_h,
        alt: alt.to_string(),
        path: path.to_string(),
    }));
    *cursor_y += box_h + 8.0;
}
