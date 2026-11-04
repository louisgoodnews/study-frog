//! PDF content stream generator converting layout primitives to graphic operators.

use super::palette::{
    COLOR_BG_CODE, COLOR_BG_INLINE_CODE, COLOR_BORDER_SUBTLE, COLOR_MUTED, COLOR_RULE, Rgb,
    rgb_to_floats,
};
use super::pdf_fonts::select_font;
use super::primitives::{
    LayoutCodeBlock, LayoutImage, LayoutItem, LayoutPage, LayoutTable, LayoutTaskCheckbox, TextLine,
};
use super::to_unicode::{bytes_to_pdf_hex, encode_text_winansi};

fn push_rect_fill(s: &mut String, x: f32, y_pdf: f32, w: f32, h: f32, color: Rgb) {
    let (r, g, b) = rgb_to_floats(color);
    s.push_str(&format!(
        "q {r:.3} {g:.3} {b:.3} rg {x:.2} {y_pdf:.2} {w:.2} {h:.2} re f Q\n"
    ));
}

#[allow(clippy::too_many_arguments)]
fn push_rect_stroke(s: &mut String, x: f32, y_pdf: f32, w: f32, h: f32, color: Rgb, lw: f32) {
    let (r, g, b) = rgb_to_floats(color);
    s.push_str(&format!(
        "q {lw:.2} w {r:.3} {g:.3} {b:.3} RG {x:.2} {y_pdf:.2} {w:.2} {h:.2} re S Q\n"
    ));
}

#[allow(clippy::too_many_arguments)]
fn push_line_stroke(s: &mut String, x1: f32, y1: f32, x2: f32, y2: f32, color: Rgb, lw: f32) {
    let (r, g, b) = rgb_to_floats(color);
    s.push_str(&format!(
        "q {lw:.2} w {r:.3} {g:.3} {b:.3} RG {x1:.2} {y1:.2} m {x2:.2} {y2:.2} l S Q\n"
    ));
}

fn emit_text_line(s: &mut String, line: &TextLine, page_h: f32) {
    let mut cur_x = line.x;
    let y_base = page_h - (line.y + line.baseline);

    for run in &line.runs {
        let (r, g, b) = rgb_to_floats(run.color);
        let font_tag = select_font(run.bold, run.italic, run.code);

        if run.code {
            let pill_y = page_h - (line.y + line.height);
            push_rect_fill(
                s,
                cur_x - 1.0,
                pill_y,
                run.width + 2.0,
                line.height,
                COLOR_BG_INLINE_CODE,
            );
        }

        let encoded = encode_text_winansi(&run.text);
        let hex_str = bytes_to_pdf_hex(&encoded);
        s.push_str(&format!(
            "BT /{font_tag} {sz:.2} Tf {r:.3} {g:.3} {b:.3} rg 1 0 0 1 {cur_x:.2} {y_base:.2} Tm {hex_str} Tj ET\n",
            sz = run.font_size
        ));

        if run.underline {
            let uy = y_base - 1.5;
            push_line_stroke(s, cur_x, uy, cur_x + run.width, uy, run.color, 0.6);
        }
        if run.strike {
            let sy = y_base + run.font_size * 0.3;
            push_line_stroke(s, cur_x, sy, cur_x + run.width, sy, run.color, 0.6);
        }
        cur_x += run.width;
    }
}

fn emit_code_box(s: &mut String, cb: &LayoutCodeBlock, page_h: f32) {
    let y_pdf = page_h - (cb.y + cb.height);
    push_rect_fill(s, cb.x, y_pdf, cb.width, cb.height, COLOR_BG_CODE);
    push_rect_stroke(
        s,
        cb.x,
        y_pdf,
        cb.width,
        cb.height,
        COLOR_BORDER_SUBTLE,
        0.5,
    );
    for line in &cb.lines {
        emit_text_line(s, line, page_h);
    }
}

fn emit_table(s: &mut String, tbl: &LayoutTable, page_h: f32) {
    for cell in &tbl.cells {
        let y_pdf = page_h - (cell.y + cell.height);
        if let Some(bg) = cell.bg_color {
            push_rect_fill(s, cell.x, y_pdf, cell.width, cell.height, bg);
        }
        push_rect_stroke(
            s,
            cell.x,
            y_pdf,
            cell.width,
            cell.height,
            COLOR_BORDER_SUBTLE,
            0.5,
        );
        for line in &cell.lines {
            emit_text_line(s, line, page_h);
        }
    }
}

fn emit_task_checkbox(s: &mut String, tc: &LayoutTaskCheckbox, page_h: f32) {
    let y_pdf = page_h - (tc.y + tc.size);
    push_rect_stroke(s, tc.x, y_pdf, tc.size, tc.size, [70, 75, 80], 0.7);
    if tc.checked {
        let x = tc.x;
        let y = y_pdf;
        let sz = tc.size;
        push_line_stroke(
            s,
            x + sz * 0.2,
            y + sz * 0.5,
            x + sz * 0.45,
            y + sz * 0.2,
            [30, 132, 73],
            1.2,
        );
        push_line_stroke(
            s,
            x + sz * 0.45,
            y + sz * 0.2,
            x + sz * 0.85,
            y + sz * 0.8,
            [30, 132, 73],
            1.2,
        );
    }
}

fn emit_image(s: &mut String, img: &LayoutImage, page_h: f32) {
    let y_pdf = page_h - (img.y + img.height);
    push_rect_fill(s, img.x, y_pdf, img.width, img.height, COLOR_BG_CODE);
    push_rect_stroke(
        s,
        img.x,
        y_pdf,
        img.width,
        img.height,
        COLOR_BORDER_SUBTLE,
        0.5,
    );
    let label = if img.alt.is_empty() {
        format!("[Picture: {}]", img.path)
    } else {
        format!("[Picture: {}]", img.alt)
    };
    let encoded = encode_text_winansi(&label);
    let hex = bytes_to_pdf_hex(&encoded);
    let text_y = y_pdf + (img.height / 2.0) - 4.0;
    let (r, g, b) = rgb_to_floats(COLOR_MUTED);
    s.push_str(&format!(
        "BT /F1 9.00 Tf {r:.3} {g:.3} {b:.3} rg 1 0 0 1 {x:.2} {text_y:.2} Tm {hex} Tj ET\n",
        x = img.x + 10.0
    ));
}

/// Generate the full PDF content stream for a page's layout items.
pub fn build_page_content_stream(page: &LayoutPage) -> Vec<u8> {
    let mut s = String::new();
    let page_h = page.height;

    for item in &page.items {
        match item {
            LayoutItem::Text(line) => emit_text_line(&mut s, line, page_h),
            LayoutItem::Heading(h) => emit_text_line(&mut s, &h.line, page_h),
            LayoutItem::Rule(r) => {
                let y_pdf = page_h - r.y;
                push_line_stroke(
                    &mut s,
                    r.x,
                    y_pdf,
                    r.x + r.width,
                    y_pdf,
                    COLOR_RULE,
                    r.thickness,
                );
            }
            LayoutItem::QuoteBar(qb) => {
                let y_bottom = page_h - (qb.y + qb.height);
                let y_top = page_h - qb.y;
                push_line_stroke(&mut s, qb.x, y_bottom, qb.x, y_top, qb.color, qb.width);
            }
            LayoutItem::CodeBox(cb) => emit_code_box(&mut s, cb, page_h),
            LayoutItem::Table(tbl) => emit_table(&mut s, tbl, page_h),
            LayoutItem::TaskCheckbox(tc) => emit_task_checkbox(&mut s, tc, page_h),
            LayoutItem::Image(img) => emit_image(&mut s, img, page_h),
        }
    }

    s.into_bytes()
}
