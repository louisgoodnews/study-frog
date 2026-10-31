//! What is drawn over a page picture: search hits, selected words, highlights, notes and links.

use eframe::egui::{self, Color32, Rect, Vec2};
use studyfrog_config::pdf_marks::MarkKind;
use studyfrog_pdf::links::PageLink;

use crate::msg::Msg;
use crate::pdf_link_types::LinkMsg;
use crate::pdf_select::matching_words;
use crate::pdf_types::{PdfMsg, PdfView};
use crate::theme::{palette_of, tint};
use crate::view::pdf_notes;

pub fn paint_words(view: &PdfView, ui: &egui::Ui, rect: Rect, logical: f32) {
    let Some(text) = &view.text else { return };
    let p = palette_of(ui);
    let boxed = |i: usize| {
        let w = &text.words[i];
        Rect::from_min_size(
            rect.min + Vec2::new(w.x, w.y) * logical,
            Vec2::new(w.width, w.height) * logical,
        )
    };
    for i in matching_words(&text.words, &view.query) {
        ui.painter().rect_filled(
            boxed(i),
            2.0,
            Color32::from_rgba_unmultiplied(255, 214, 0, 110),
        );
    }
    for i in view.selection.iter().filter(|i| **i < text.words.len()) {
        ui.painter().rect_filled(boxed(*i), 2.0, tint(p.accent, 90));
    }
}

pub fn paint_marks(view: &PdfView, ui: &egui::Ui, rect: Rect, logical: f32) {
    for m in view.marks.iter().filter(|m| m.page == view.page) {
        let color = pdf_notes::color_of(&m.color);
        for r in &m.rects {
            let area = Rect::from_min_size(
                rect.min + Vec2::new(r.x, r.y) * logical,
                Vec2::new(r.width, r.height) * logical,
            );
            match m.kind {
                MarkKind::Highlight => {
                    ui.painter()
                        .rect_filled(area, 2.0, color.gamma_multiply(0.45));
                }
                MarkKind::Note => {
                    ui.painter().rect_filled(area, 3.0, color);
                    ui.painter().rect_stroke(
                        area,
                        3.0,
                        ui.visuals().widgets.active.fg_stroke,
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
    }
}

fn rect_of(link: &PageLink, page: Rect, logical: f32) -> Rect {
    Rect::from_min_size(
        page.min + Vec2::new(link.x, link.y) * logical,
        Vec2::new(link.width, link.height) * logical,
    )
}

/// Outline every link of the page.
pub fn paint_links(view: &PdfView, ui: &egui::Ui, page: Rect, logical: f32) {
    let Some(text) = &view.text else { return };
    let stroke = egui::Stroke::new(1.0, palette_of(ui).accent);
    for l in &text.links {
        ui.painter().rect_stroke(
            rect_of(l, page, logical),
            2.0,
            stroke,
            egui::StrokeKind::Outside,
        );
    }
}

/// A click on a link is a request to follow it; true when the click was on one.
pub fn click_link(
    view: &PdfView,
    response: &egui::Response,
    page: Rect,
    logical: f32,
    out: &mut Vec<Msg>,
) -> bool {
    let Some(text) = &view.text else { return false };
    let Some(at) = response.hover_pos() else {
        return false;
    };
    let Some(link) = text
        .links
        .iter()
        .find(|l| rect_of(l, page, logical).contains(at))
    else {
        return false;
    };
    response.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    if response.clicked() {
        out.push(Msg::Pdf(PdfMsg::Link(LinkMsg::Click(link.target.clone()))));
    }
    true
}
