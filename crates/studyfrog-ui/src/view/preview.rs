//! Drawing the preview blocks. The blocks are parsed once when the text changes, never per frame.

use eframe::egui::{self, Align, Layout, RichText, vec2};
use studyfrog_md::blocks::{Block, Link, ListItem, Span};

use crate::msg::Msg;
use crate::typo::{self, Tier};
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
        Block::Paragraph(spans) => {
            ui.scope(|ui| spans_line(ui, spans, None, out));
        }
        Block::Code { text, tokens, .. } => rich::code(ui, text, tokens),
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
            typo::line_height(typo::body_size(ui)),
        );
        ui.allocate_ui_with_layout(row, Layout::left_to_right(Align::Center), |ui| {
            ui.label(mark);
        });
        ui.vertical(|ui| draw_all(ui, &item.blocks, out, toc));
    });
}

/// Headings of levels 1 and 2 are in the heading tier, the others in the body tier (strong), so a document
/// keeps to the four sizes of the type scale (`typo`).
fn heading_size(ui: &egui::Ui, level: u8) -> f32 {
    typo::size(
        ui,
        if level <= 2 {
            Tier::Heading
        } else {
            Tier::Body
        },
    )
}

/// The size of a paragraph's or heading's text.
fn text_size(ui: &egui::Ui, heading: Option<u8>) -> f32 {
    heading.map_or_else(|| typo::body_size(ui), |l| heading_size(ui, l))
}

pub(super) fn styled(ui: &egui::Ui, span: &Span, heading: Option<u8>) -> RichText {
    let size = text_size(ui, heading);
    let mut text = typo::prose(ui, &span.text, size);
    if heading.is_some() {
        text = text.strong();
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
    let measure = typo::measure(ui);
    let line = typo::line_height(text_size(ui, heading));
    // The row height is taken when the wrapping layout starts, so it is set on a scope around it: then the first
    // line of a paragraph is exactly as tall as the others.
    ui.spacing_mut().interact_size.y = line;
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.horizontal_wrapped(|ui| {
        // Prose is never wider than the reading measure.
        ui.set_max_width(ui.available_width().min(measure));
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
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
