//! Drawing the preview blocks. The blocks are parsed once when the text changes, never per frame.

use eframe::egui::{self, RichText};
use studyfrog_md::blocks::{Block, Link, ListItem, Span};

use crate::msg::Msg;

/// Draw `blocks`; clicking a wikilink pushes [`Msg::OpenLink`].
pub fn draw_blocks(ui: &mut egui::Ui, blocks: &[Block], out: &mut Vec<Msg>) {
    for block in blocks {
        draw_block(ui, block, out);
    }
}

fn draw_block(ui: &mut egui::Ui, block: &Block, out: &mut Vec<Msg>) {
    match block {
        Block::Heading { level, spans } => spans_line(ui, spans, Some(*level), out),
        Block::Paragraph(spans) => spans_line(ui, spans, None, out),
        Block::Code { text, .. } => {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(text.trim_end()).monospace()).wrap());
            });
        }
        Block::Quote(inner) => {
            ui.indent("quote", |ui| draw_blocks(ui, inner, out));
        }
        Block::List {
            ordered,
            start,
            items,
        } => {
            for (i, item) in items.iter().enumerate() {
                list_item(ui, item, marker(*ordered, *start, i), out);
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

fn list_item(ui: &mut egui::Ui, item: &ListItem, marker: String, out: &mut Vec<Msg>) {
    ui.horizontal_top(|ui| {
        let mark = match item.checked {
            Some(true) => "☑".to_owned(),
            Some(false) => "☐".to_owned(),
            None => marker,
        };
        ui.label(mark);
        ui.vertical(|ui| draw_blocks(ui, &item.blocks, out));
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

fn styled(span: &Span, heading: Option<u8>) -> RichText {
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
    text
}

/// One paragraph or heading: spans flow and wrap, links are clickable.
fn spans_line(ui: &mut egui::Ui, spans: &[Span], heading: Option<u8>, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for span in spans {
            let text = styled(span, heading);
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
                Some(Link::Url(url)) => {
                    ui.hyperlink_to(text, url);
                }
                None => {
                    ui.label(text);
                }
            }
        }
    });
}
