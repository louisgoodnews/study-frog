//! One page of a PDF: its picture as a texture, the words found by the search, and the words the
//! user selects by dragging a rectangle.

use std::sync::Arc;

use eframe::egui::{self, Color32, Pos2, Rect, ScrollArea, Sense, Vec2};
use studyfrog_pdf::render::{PageRect, Pixels};

use crate::msg::Msg;
use crate::pdf_extract_types::ExtractMsg;
use crate::pdf_mark_types::MarkMsg;
use crate::pdf_select::words_in_rect;
use crate::pdf_types::{PageKey, PdfMsg, PdfView, Zoom};
use crate::view::pdf_paint;
pub use crate::view::pdf_texture::{mark_used, prune, texture};

/// Logical pixels per point at 100 %.
const POINT_PX: f32 = 96.0 / 72.0;
const MARGIN: f32 = 24.0;

/// The picture for `key`, else any other picture of the same page.
pub fn picture_for(view: &PdfView, key: PageKey) -> Option<Arc<Pixels>> {
    let exact = view.pictures.iter().find(|(k, _)| *k == key);
    let same_page = view.pictures.iter().rev().find(|(k, _)| k.page == key.page);
    exact.or(same_page).map(|(_, px)| px.clone())
}

/// The largest scale (thousandths, a multiple of 50) at which a page of `page` points fits into a
/// texture of `max_side` texels a side and into the renderer's pixel budget. Above it the page
/// is drawn at this scale and stretched, rather than failing to render or to upload.
pub fn max_milli(page: (f32, f32), max_side: usize) -> f32 {
    let (w, h) = (page.0.max(1.0), page.1.max(1.0));
    let by_side = (max_side.saturating_sub(1) as f32) / w.max(h);
    let by_pixels = (studyfrog_pdf::limits::MAX_RENDER_PIXELS as f32 / (w * h)).sqrt();
    ((by_side.min(by_pixels) * 1000.0 / 50.0).floor() * 50.0).max(50.0)
}

/// The scale (thousandths of a pixel per point) and the size on screen in logical pixels.
fn layout(view: &PdfView, avail: Vec2, ppp: f32, page: (f32, f32), max_side: usize) -> (u32, f32) {
    let wide = (avail.x - MARGIN).max(100.0) / page.0.max(1.0);
    let high = (avail.y - MARGIN).max(100.0) / page.1.max(1.0);
    let logical = match view.zoom {
        Zoom::FitWidth => wide,
        Zoom::FitPage => wide.min(high),
        Zoom::Percent(p) => p as f32 / 100.0 * POINT_PX,
    };
    let milli = ((logical * ppp * 20.0).round() * 50.0).clamp(50.0, 16000.0);
    (milli.min(max_milli(page, max_side)) as u32, logical)
}

fn page_rect_of(rect: Rect, a: Pos2, b: Pos2, logical: f32) -> PageRect {
    let (lo, hi) = (a.min(b), a.max(b));
    PageRect {
        x: (lo.x - rect.min.x) / logical,
        y: (lo.y - rect.min.y) / logical,
        width: (hi.x - lo.x) / logical,
        height: (hi.y - lo.y) / logical,
    }
}

fn place(
    view: &PdfView,
    response: &egui::Response,
    rect: Rect,
    logical: f32,
    out: &mut Vec<Msg>,
) -> bool {
    let Some(at) = response
        .interact_pointer_pos()
        .filter(|_| view.placing_note && response.clicked())
    else {
        return false;
    };
    let p = (at - rect.min) / logical;
    out.push(Msg::Pdf(PdfMsg::Marks(MarkMsg::PlaceNote(p.x, p.y))));
    true
}

/// In snapshot mode a dragged rectangle is shown and, when let go, saved as a picture.
fn snapshot(
    view: &PdfView,
    ui: &egui::Ui,
    response: &egui::Response,
    rect: Rect,
    logical: f32,
    out: &mut Vec<Msg>,
) -> bool {
    if !view.extract.snapshot_mode {
        return false;
    }
    let drag = response
        .interact_pointer_pos()
        .zip(ui.input(|i| i.pointer.press_origin()));
    let Some((now, start)) = drag.filter(|_| response.dragged() || response.drag_stopped()) else {
        return true;
    };
    let area = page_rect_of(rect, start, now, logical);
    ui.painter().rect_stroke(
        Rect::from_two_pos(start, now),
        0.0,
        ui.visuals().selection.stroke,
        egui::StrokeKind::Outside,
    );
    if response.drag_stopped() {
        out.push(Msg::Pdf(PdfMsg::Extract(ExtractMsg::Snapshot(area))));
    }
    true
}

fn select(view: &PdfView, response: &egui::Response, rect: Rect, logical: f32, out: &mut Vec<Msg>) {
    let Some(text) = &view.text else { return };
    let drag = response
        .interact_pointer_pos()
        .zip(response.ctx.input(|i| i.pointer.press_origin()));
    let words = match (response.dragged() || response.drag_stopped(), drag) {
        (true, Some((now, start))) => {
            words_in_rect(&text.words, &page_rect_of(rect, start, now, logical))
        }
        _ if response.clicked() => Vec::new(),
        _ => return,
    };
    if words != view.selection {
        out.push(Msg::Pdf(PdfMsg::Select(words)));
    }
}

/// Draw the shown page, scrollable, and ask for its picture when it is missing.
pub fn draw(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(size) = view
        .info
        .as_ref()
        .and_then(|i| i.pages.get(view.page))
        .copied()
    else {
        return;
    };
    prune(ui.ctx(), view);
    let (milli, logical) = layout(
        view,
        ui.available_size(),
        ui.ctx().pixels_per_point(),
        (size.width, size.height),
        ui.input(|i| i.max_texture_side),
    );
    let key = PageKey {
        page: view.page,
        scale_milli: milli,
    };
    ScrollArea::both()
        .id_salt("pdf-page-scroll")
        .show(ui, |ui| {
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(size.width, size.height) * logical,
                Sense::click_and_drag(),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::Role::Image,
                    true,
                    format!("Page {}", view.page + 1),
                )
            });
            match picture_for(view, key) {
                Some(px) => {
                    let tex = texture(ui.ctx(), &view.path, key, &px);
                    egui::Image::new(&tex).paint_at(ui, rect);
                    mark_used(view, key, out);
                }
                None => {
                    ui.painter().rect_filled(rect, 0.0, Color32::WHITE);
                    if view.requested.contains(&key) {
                        ui.ctx().request_repaint();
                    }
                }
            }
            if !view.pictures.iter().any(|(k, _)| *k == key) {
                out.push(Msg::Pdf(PdfMsg::Need(key)));
            }
            pdf_paint::paint_marks(view, ui, rect, logical);
            pdf_paint::paint_words(view, ui, rect, logical);
            pdf_paint::paint_links(view, ui, rect, logical);
            if !pdf_paint::click_link(view, &response, rect, logical, out)
                && !place(view, &response, rect, logical, out)
                && !snapshot(view, ui, &response, rect, logical, out)
            {
                select(view, &response, rect, logical, out);
            }
            page_text(view, ui);
        });
}

/// The page's text as plain, selectable text, for screen readers and for copying.
fn page_text(view: &PdfView, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Text of this page")
        .id_salt("pdf-page-text")
        .show(ui, |ui| match view.text.as_ref() {
            Some(t) if t.image_only => {
                ui.weak("This page is a picture and has no text.");
            }
            Some(t) if t.text.trim().is_empty() => {
                ui.weak("This page has no text.");
            }
            Some(t) => {
                ui.add(egui::Label::new(t.text.trim()).selectable(true));
            }
            None => {
                ui.weak("Reading the page ...");
            }
        });
}
