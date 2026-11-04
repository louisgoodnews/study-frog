//! Layout and pagination for tables with zebra striping and header repetition.

use studyfrog_md::blocks::{Align, Row, Span};

use super::palette::{COLOR_BG_CODE, COLOR_BG_TABLE_HEADER, Rgb};
use super::primitives::{LayoutItem, LayoutPage, LayoutTable, LayoutTableCell, TextLine};
use super::wrap::wrap_spans;

const CELL_PAD_X: f32 = 6.0;
const CELL_PAD_Y: f32 = 4.0;
const TABLE_FONT_SIZE: f32 = 8.5;
const TABLE_LINE_HEIGHT: f32 = 11.0;

fn calculate_col_widths(num_cols: usize, total_w: f32) -> Vec<f32> {
    if num_cols == 0 {
        return Vec::new();
    }
    let col_w = total_w / num_cols as f32;
    vec![col_w; num_cols]
}

fn align_line_x(x: f32, inner_w: f32, align: Align, line: &mut TextLine) {
    let line_w: f32 = line.runs.iter().map(|r| r.width).sum();
    let offset = match align {
        Align::Center => (inner_w - line_w).max(0.0) / 2.0,
        Align::Right => (inner_w - line_w).max(0.0),
        Align::Left | Align::None => 0.0,
    };
    line.x = x + offset;
}

#[allow(clippy::too_many_arguments)]
fn layout_row_cells(
    row: &Row,
    aligns: &[Align],
    col_widths: &[f32],
    start_x: f32,
    y: f32,
    is_header: bool,
    bg_color: Option<Rgb>,
) -> (Vec<LayoutTableCell>, f32) {
    let mut cells = Vec::new();
    let mut cell_lines_list = Vec::new();
    let mut max_lines = 1;

    for (c, col_w) in col_widths.iter().enumerate() {
        let inner_w = (col_w - CELL_PAD_X * 2.0).max(10.0);
        let cell_spans: &[Span] = row.get(c).map(|v| v.as_slice()).unwrap_or(&[]);
        let mut spans_with_bold = cell_spans.to_vec();
        if is_header {
            for s in &mut spans_with_bold {
                s.style.bold = true;
            }
        }
        let lines = wrap_spans(
            &spans_with_bold,
            inner_w,
            TABLE_FONT_SIZE,
            TABLE_LINE_HEIGHT,
        );
        max_lines = max_lines.max(lines.len().max(1));
        cell_lines_list.push(lines);
    }

    let row_h = max_lines as f32 * TABLE_LINE_HEIGHT + CELL_PAD_Y * 2.0;
    let mut cur_x = start_x;

    for (c, col_w) in col_widths.iter().enumerate() {
        let inner_w = (col_w - CELL_PAD_X * 2.0).max(10.0);
        let align = aligns.get(c).copied().unwrap_or(Align::Left);
        let mut positioned_lines = cell_lines_list[c].clone();
        let mut line_y = y + CELL_PAD_Y;
        for line in &mut positioned_lines {
            line.y = line_y;
            align_line_x(cur_x + CELL_PAD_X, inner_w, align, line);
            line_y += TABLE_LINE_HEIGHT;
        }

        cells.push(LayoutTableCell {
            x: cur_x,
            y,
            width: *col_w,
            height: row_h,
            lines: positioned_lines,
            is_header,
            bg_color,
        });
        cur_x += *col_w;
    }

    (cells, row_h)
}

/// Layout a table onto pages, cleanly splitting across page boundaries with repeated header.
#[allow(clippy::too_many_arguments)]
pub fn layout_table_block(
    aligns: &[Align],
    header: &Row,
    rows: &[Row],
    page: &mut LayoutPage,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    (content_x, content_top, content_w, content_h): (f32, f32, f32, f32),
) {
    let num_cols = aligns
        .len()
        .max(header.len())
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
    if num_cols == 0 {
        return;
    }
    let col_widths = calculate_col_widths(num_cols, content_w);

    let (_hdr_cells, hdr_h) = layout_row_cells(
        header,
        aligns,
        &col_widths,
        content_x,
        *cursor_y,
        true,
        Some(COLOR_BG_TABLE_HEADER),
    );

    let mut current_table_cells = Vec::new();
    let mut table_start_y = *cursor_y;

    if content_h - *cursor_y < hdr_h + 20.0 {
        pages.push(std::mem::replace(
            page,
            super::primitives::new_layout_page(pages.len() + 1, page.width, page.height),
        ));
        *cursor_y = content_top;
        table_start_y = content_top;
    }

    let (hdr_cells, hdr_h) = layout_row_cells(
        header,
        aligns,
        &col_widths,
        content_x,
        *cursor_y,
        true,
        Some(COLOR_BG_TABLE_HEADER),
    );
    current_table_cells.extend(hdr_cells);
    *cursor_y += hdr_h;

    for (row_idx, row) in rows.iter().enumerate() {
        let bg = if row_idx % 2 == 1 {
            Some(COLOR_BG_CODE)
        } else {
            None
        };
        let (_temp_cells, row_h) =
            layout_row_cells(row, aligns, &col_widths, content_x, *cursor_y, false, bg);

        if *cursor_y + row_h > content_h {
            if !current_table_cells.is_empty() {
                page.items.push(LayoutItem::Table(LayoutTable {
                    x: content_x,
                    y: table_start_y,
                    width: content_w,
                    height: *cursor_y - table_start_y,
                    cells: std::mem::take(&mut current_table_cells),
                }));
            }
            pages.push(std::mem::replace(
                page,
                super::primitives::new_layout_page(pages.len() + 1, page.width, page.height),
            ));
            *cursor_y = content_top;
            table_start_y = content_top;

            let (rep_hdr, rh_h) = layout_row_cells(
                header,
                aligns,
                &col_widths,
                content_x,
                *cursor_y,
                true,
                Some(COLOR_BG_TABLE_HEADER),
            );
            current_table_cells.extend(rep_hdr);
            *cursor_y += rh_h;
        }

        let (row_cells, placed_h) =
            layout_row_cells(row, aligns, &col_widths, content_x, *cursor_y, false, bg);
        current_table_cells.extend(row_cells);
        *cursor_y += placed_h;
    }

    if !current_table_cells.is_empty() {
        page.items.push(LayoutItem::Table(LayoutTable {
            x: content_x,
            y: table_start_y,
            width: content_w,
            height: *cursor_y - table_start_y,
            cells: current_table_cells,
        }));
    }
    *cursor_y += 8.0;
}
