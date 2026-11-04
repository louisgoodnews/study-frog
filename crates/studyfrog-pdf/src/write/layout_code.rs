//! Layout and pagination for syntax-highlighted code blocks.

use studyfrog_md::highlight::{Kind, Token};

use super::metrics::measure_str;
use super::palette::token_kind_color;
use super::primitives::{LayoutCodeBlock, LayoutItem, LayoutPage, StyledRun, TextLine};

const CODE_FONT_SIZE: f32 = 8.5;
const CODE_LINE_HEIGHT: f32 = 11.0;
const CODE_PAD_X: f32 = 8.0;
const CODE_PAD_Y: f32 = 6.0;

fn split_token_runs(text: &str, tokens: &[Token]) -> Vec<Vec<StyledRun>> {
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut offset = 0;

    let mut process_span = |slice: &str, kind: Kind| {
        let is_kw = kind == Kind::Keyword;
        let is_cm = kind == Kind::Comment;
        let color = token_kind_color(kind);
        let mut parts = slice.split('\n').peekable();
        while let Some(part) = parts.next() {
            if !part.is_empty() {
                current_line.push(StyledRun {
                    text: part.to_string(),
                    bold: is_kw,
                    italic: is_cm,
                    underline: false,
                    strike: false,
                    code: true,
                    color,
                    font_size: CODE_FONT_SIZE,
                    width: measure_str(part, CODE_FONT_SIZE, is_kw, true),
                    link: None,
                });
            }
            if parts.peek().is_some() {
                lines.push(std::mem::take(&mut current_line));
            }
        }
    };

    if tokens.is_empty() {
        process_span(text, Kind::Plain);
    } else {
        for token in tokens {
            let end = (offset + token.len).min(text.len());
            let slice = &text[offset..end];
            process_span(slice, token.kind);
            offset = end;
        }
    }
    if !current_line.is_empty() || lines.is_empty() {
        lines.push(current_line);
    }
    lines
}

/// Convert raw code runs into wrapped lines that do not exceed the box inner width.
fn wrap_code_lines(raw_lines: Vec<Vec<StyledRun>>, inner_w: f32) -> Vec<Vec<StyledRun>> {
    let mut wrapped = Vec::new();
    for line in raw_lines {
        let total_w: f32 = line.iter().map(|r| r.width).sum();
        if total_w <= inner_w {
            wrapped.push(line);
            continue;
        }
        let mut cur_line = Vec::new();
        let mut cur_w = 0.0;
        for run in line {
            if cur_w + run.width <= inner_w {
                cur_w += run.width;
                cur_line.push(run);
            } else {
                for c in run.text.chars() {
                    let cw = measure_str(&c.to_string(), run.font_size, run.bold, true);
                    if cur_w + cw > inner_w && !cur_line.is_empty() {
                        wrapped.push(std::mem::take(&mut cur_line));
                        cur_w = 0.0;
                    }
                    if let Some(last) = cur_line.last_mut() {
                        last.text.push(c);
                        last.width += cw;
                    } else {
                        let mut new_run = run.clone();
                        new_run.text = c.to_string();
                        new_run.width = cw;
                        cur_line.push(new_run);
                    }
                    cur_w += cw;
                }
            }
        }
        if !cur_line.is_empty() {
            wrapped.push(cur_line);
        }
    }
    wrapped
}

/// Layout a code block onto the page, splitting across page boundaries if necessary.
#[allow(clippy::too_many_arguments)]
pub fn layout_code_block(
    lang: &str,
    text: &str,
    tokens: &[Token],
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    (content_x, content_top, content_w, content_h): (f32, f32, f32, f32),
) {
    let inner_w = content_w - CODE_PAD_X * 2.0;
    let raw_runs = split_token_runs(text, tokens);
    let code_lines = wrap_code_lines(raw_runs, inner_w);
    let mut line_idx = 0;
    let mut is_cont = false;

    while line_idx < code_lines.len() {
        let space_left = content_h - *cursor_y;
        if space_left < CODE_PAD_Y * 2.0 + CODE_LINE_HEIGHT {
            pages.push(std::mem::replace(
                page,
                super::primitives::new_layout_page(pages.len() + 1, page.width, page.height),
            ));
            *cursor_y = content_top;
            is_cont = true;
        }

        let avail_lines = (((content_h - *cursor_y - CODE_PAD_Y * 2.0) / CODE_LINE_HEIGHT).floor()
            as usize)
            .max(1);
        let end_idx = (line_idx + avail_lines).min(code_lines.len());
        let slice = &code_lines[line_idx..end_idx];

        let box_y = *cursor_y;
        let mut placed_lines = Vec::new();
        let mut cur_line_y = box_y + CODE_PAD_Y;

        for line_runs in slice {
            placed_lines.push(TextLine {
                x: content_x + CODE_PAD_X,
                y: cur_line_y,
                runs: line_runs.clone(),
                height: CODE_LINE_HEIGHT,
                baseline: CODE_LINE_HEIGHT * 0.75,
            });
            cur_line_y += CODE_LINE_HEIGHT;
        }

        let box_h = cur_line_y - box_y + CODE_PAD_Y;
        page.items.push(LayoutItem::CodeBox(LayoutCodeBlock {
            x: content_x,
            y: box_y,
            width: content_w,
            height: box_h,
            lines: placed_lines,
            lang: lang.to_string(),
            continued: is_cont,
        }));

        *cursor_y = box_y + box_h + 8.0;
        line_idx = end_idx;
        is_cont = true;
    }
}
