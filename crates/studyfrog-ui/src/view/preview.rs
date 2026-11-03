//! Drawing the preview blocks. The blocks are parsed once when the text changes, never per frame.

use eframe::egui::{self, Align, Layout, RichText, vec2};
use studyfrog_md::blocks::{Block, Link, ListItem, Span};

use crate::msg::Msg;
use crate::view::preview_rich as rich;

/// Draw `blocks`; clicking a wikilink pushes [`Msg::OpenLink`].
pub fn draw_blocks(ui: &mut egui::Ui, blocks: &[Block], out: &mut Vec<Msg>) {
    let toc = if blocks.iter().any(|b| matches!(b, Block::Toc)) {
        rich::headings(blocks)
    } else {
        Vec::new()
    };
    draw_all(ui, blocks, out, &toc);
}

fn draw_all(ui: &mut egui::Ui, blocks: &[Block], out: &mut Vec<Msg>, toc: &[rich::Heading]) {
    for block in blocks {
        draw_block(ui, block, out, toc);
    }
}

fn draw_block(ui: &mut egui::Ui, block: &Block, out: &mut Vec<Msg>, toc: &[rich::Heading]) {
    match block {
        Block::Heading { level, spans } => rich::heading(ui, (*level, spans), out),
        Block::Table {
            aligns,
            header,
            rows,
        } => rich::table(ui, (aligns, header, rows), out),
        Block::Toc => rich::toc(ui, toc),
        Block::Paragraph(spans) => spans_line(ui, spans, None, out),
        Block::Code { text, .. } => {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(text.trim_end()).monospace()).wrap());
            });
        }
        Block::Quote(inner) => {
            ui.indent("quote", |ui| draw_all(ui, inner, out, toc));
        }
        Block::List {
            ordered,
            start,
            items,
        } => {
            for (i, item) in items.iter().enumerate() {
                list_item(ui, item, marker(*ordered, *start, i), (out, toc));
            }
        }
        Block::Rule => {
            ui.separator();
        }
    }
}

fn marker(ordered: bool, start: u64, index: usize) -> String {
    if ordered {
        format!("{}.", start + index as u64)
    } else {
        "•".to_owned()
    }
}

fn list_item(
    ui: &mut egui::Ui,
    item: &ListItem,
    marker: String,
    (out, toc): (&mut Vec<Msg>, &[rich::Heading]),
) {
    ui.horizontal_top(|ui| {
        let mark = match item.checked {
            Some(true) => "☑".to_owned(),
            Some(false) => "☐".to_owned(),
            None => marker,
        };
        // A paragraph's first line sits in a row at least as tall as a button, with its text in the middle of
        // it; the marker gets a row of the same height so that it is level with that line.
        let row = vec2(
            ui.spacing().interact_size.x.min(28.0),
            ui.spacing().interact_size.y,
        );
        ui.allocate_ui_with_layout(row, Layout::left_to_right(Align::Center), |ui| {
            ui.label(mark);
        });
        ui.vertical(|ui| draw_all(ui, &item.blocks, out, toc));
    });
}

fn heading_size(level: u8) -> f32 {
    match level {
        1 => 26.0,
        2 => 22.0,
        3 => 19.0,
        _ => 16.0,
    }
}

pub(super) fn styled(ui: &egui::Ui, span: &Span, heading: Option<u8>) -> RichText {
    let mut text = RichText::new(&span.text);
    if let Some(level) = heading {
        text = text.size(heading_size(level)).strong();
    }
    let s = span.style;
    if s.bold {
        text = text.strong();
    }
    if s.italic {
        text = text.italics();
    }
    if s.strike {
        text = text.strikethrough();
    }
    if s.code {
        text = text.code();
    }
    if s.underline {
        text = text.underline();
    }
    if let Some(hue) = s.color {
        text = text.color(rich::hue_color_in(ui, hue));
    }
    text
}

/// One paragraph or heading: spans flow and wrap, links are clickable.
pub(super) fn spans_line(
    ui: &mut egui::Ui,
    spans: &[Span],
    heading: Option<u8>,
    out: &mut Vec<Msg>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for span in spans {
            let text = styled(ui, span, heading);
            match &span.link {
                Some(Link::Wiki(target)) => {
                    if ui
                        .link(text)
                        .on_hover_text(format!("Open \"{target}\""))
                        .clicked()
                    {
                        out.push(Msg::OpenLink(target.clone()));
                    }
                }
                Some(Link::Url(url)) => match crate::pdf_link::pdf_target(url) {
                    Some((path, page)) => pdf_link(ui, text, (path, page), out),
                    None => {
                        ui.hyperlink_to(text, url);
                    }
                },
                Some(Link::Image(path)) => match crate::pdf_link::pdf_target(path) {
                    Some((p, page)) => {
                        let name = if span.text.is_empty() { &p } else { &span.text };
                        pdf_link(
                            ui,
                            egui::RichText::new(format!("PDF: {name}, page {}", page + 1)),
                            (p.clone(), page),
                            out,
                        );
                    }
                    None => crate::view::image::draw_image(ui, &span.text, path),
                },
                None => {
                    ui.label(text);
                }
            }
        }
    });
}

fn pdf_link(
    ui: &mut egui::Ui,
    text: egui::RichText,
    (path, page): (String, usize),
    out: &mut Vec<Msg>,
) {
    if ui
        .link(text)
        .on_hover_text(format!("Open {path}, page {}", page + 1))
        .clicked()
    {
        out.push(Msg::Pdf(crate::pdf_types::PdfMsg::Open(path, page)));
    }
}
