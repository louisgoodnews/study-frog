//! Parts of the preview beyond paragraphs and lists: tables, the table of contents, headings that can be scrolled to,
//! and the colours of coloured text.

use eframe::egui::{self, Align, Id, Layout, RichText, TextFormat, TextStyle, text::LayoutJob};
use studyfrog_md::blocks::{Align as Column, Row, Span, plain_text};
use studyfrog_md::highlight::{Kind, Token};
use studyfrog_md::hue::Hue;

use super::preview::spans_line;
use crate::msg::Msg;
use crate::theme::{look_of, palette_of};
use crate::theme_hue::hue_color;

/// A heading for the table of contents: level and text.
pub type Heading = (u8, String);

/// The colour of a text hue in the theme `ui` is drawn in.
pub fn hue_color_in(ui: &egui::Ui, hue: Hue) -> egui::Color32 {
    let look = look_of(ui.ctx());
    hue_color(
        hue,
        ui.visuals().dark_mode,
        look.high_contrast,
        &palette_of(ui),
    )
}

/// The headings of the document, in order, for `[TOC]`.
pub fn headings(blocks: &[studyfrog_md::blocks::Block]) -> Vec<Heading> {
    use studyfrog_md::blocks::Block;
    let mut out = Vec::new();
    for b in blocks {
        match b {
            Block::Heading { level, spans } => out.push((*level, plain_text(spans))),
            Block::Quote(inner) => out.extend(headings(inner)),
            _ => {}
        }
    }
    out
}

/// The heading the table of contents asked to scroll to. Temporary data is keyed by its type, so the null id will do.
#[derive(Clone)]
struct ScrollTo(String);

/// A heading; when the table of contents asked for it, the preview scrolls here.
pub fn heading(ui: &mut egui::Ui, (level, spans): (u8, &[Span]), out: &mut Vec<Msg>) {
    let text = plain_text(spans);
    let r = ui
        .scope(|ui| spans_line(ui, spans, Some(level), out))
        .response;
    let wanted = ui.data(|d| d.get_temp::<ScrollTo>(Id::NULL));
    if wanted.is_some_and(|w| w.0 == text) {
        r.scroll_to_me(Some(Align::TOP));
        ui.data_mut(|d| d.remove::<ScrollTo>(Id::NULL));
    }
}

/// The table of contents: the headings with their indent; clicking one scrolls to it.
pub fn toc(ui: &mut egui::Ui, headings: &[Heading]) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new("Contents").strong());
        if headings.is_empty() {
            ui.label(
                RichText::new("This note has no headings yet.").color(palette_of(ui).text_weak),
            );
        }
        let top = headings.iter().map(|(l, _)| *l).min().unwrap_or(1);
        for (level, text) in headings {
            ui.horizontal(|ui| {
                ui.add_space(f32::from(level - top) * 14.0);
                if ui.link(text).clicked() {
                    ui.data_mut(|d| d.insert_temp(Id::NULL, ScrollTo(text.clone())));
                    ui.ctx().request_repaint();
                }
            });
        }
    });
}

fn alignment(c: Column) -> Align {
    match c {
        Column::Center => Align::Center,
        Column::Right => Align::Max,
        Column::Left | Column::None => Align::Min,
    }
}

/// A centred or right-aligned cell is one laid-out text, so its alignment holds when it wraps.
fn aligned_cell(ui: &mut egui::Ui, spans: &[Span], align: Align) {
    let mut job = LayoutJob {
        halign: align,
        ..LayoutJob::default()
    };
    job.wrap.max_width = ui.available_width();
    for span in spans {
        let mut format = TextFormat::simple(
            TextStyle::Body.resolve(ui.style()),
            ui.visuals().text_color(),
        );
        format.underline = if span.style.underline {
            egui::Stroke::new(1.0, format.color)
        } else {
            egui::Stroke::NONE
        };
        format.italics = span.style.italic;
        if let Some(hue) = span.style.color {
            format.color = hue_color_in(ui, hue);
        }
        job.append(&span.text, 0.0, format);
    }
    ui.label(job);
}

fn cell(ui: &mut egui::Ui, spans: &[Span], (column, header): (Column, bool), out: &mut Vec<Msg>) {
    let align = alignment(column);
    if align == Align::Min && !header {
        spans_line(ui, spans, None, out);
    } else if align == Align::Min {
        ui.label(RichText::new(plain_text(spans)).strong());
    } else {
        ui.with_layout(Layout::top_down(align), |ui| {
            if header {
                ui.label(RichText::new(plain_text(spans)).strong());
            } else {
                aligned_cell(ui, spans, align);
            }
        });
    }
}

/// A pipe table: a header row in bold, then the rows, striped.
pub fn table(
    ui: &mut egui::Ui,
    (aligns, header, rows): (&[Column], &Row, &[Row]),
    out: &mut Vec<Msg>,
) {
    let id = ui.next_auto_id();
    egui::Grid::new(id)
        .striped(true)
        .spacing([14.0, 4.0])
        .show(ui, |ui| {
            let column = |i: usize| aligns.get(i).copied().unwrap_or(Column::None);
            for (i, c) in header.iter().enumerate() {
                cell(ui, c, (column(i), true), out);
            }
            ui.end_row();
            for row in rows {
                for (i, c) in row.iter().enumerate() {
                    cell(ui, c, (column(i), false), out);
                }
                ui.end_row();
            }
        });
    ui.add_space(4.0);
}

/// The colour of a kind of code on the theme `ui` is drawn in; plain code and high contrast use the text colour.
fn token_color(ui: &egui::Ui, kind: Kind) -> egui::Color32 {
    let hue = match kind {
        Kind::Keyword => Hue::Purple,
        Kind::String => Hue::Green,
        Kind::Comment => Hue::Grey,
        Kind::Number => Hue::Orange,
        Kind::Plain => return ui.visuals().text_color(),
    };
    hue_color_in(ui, hue)
}

/// A code block: monospace in a frame, with the highlighter's colours when the language is known. Comments are
/// also italic, and every character of the code is still there to read, so colour is never the only carrier.
pub fn code(ui: &mut egui::Ui, text: &str, tokens: &[Token]) {
    let text = text.trim_end();
    egui::Frame::group(ui.style()).show(ui, |ui| {
        if tokens.is_empty() {
            ui.add(egui::Label::new(RichText::new(text).monospace()).wrap());
            return;
        }
        let mut job = LayoutJob::default();
        job.wrap.max_width = ui.available_width();
        let font = TextStyle::Monospace.resolve(ui.style());
        let mut at = 0;
        for t in tokens {
            let end = (at + t.len).min(text.len());
            if at >= end {
                break;
            }
            let mut format = TextFormat::simple(font.clone(), token_color(ui, t.kind));
            format.italics = t.kind == Kind::Comment;
            job.append(&text[at..end], 0.0, format);
            at = end;
        }
        ui.add(egui::Label::new(job).wrap());
    });
}
