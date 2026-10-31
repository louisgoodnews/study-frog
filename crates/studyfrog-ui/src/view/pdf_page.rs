//! One page of a PDF: its picture as a texture, the words found by the search, and the words the
//! user selects by dragging a rectangle.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, ColorImage, Pos2, Rect, ScrollArea, Sense, TextureHandle, TextureOptions, Vec2,
};
use studyfrog_pdf::render::{PageRect, Pixels};

use crate::msg::Msg;
use crate::pdf_select::{matching_words, words_in_rect};
use crate::pdf_types::{PageKey, PdfMsg, PdfView, Zoom};
use crate::theme::{palette_of, tint};

/// Logical pixels per point at 100 %.
const POINT_PX: f32 = 96.0 / 72.0;
const MARGIN: f32 = 24.0;

/// The picture for `key`, else any other picture of the same page.
pub fn picture_for(view: &PdfView, key: PageKey) -> Option<Arc<Pixels>> {
    let exact = view.pictures.iter().find(|(k, _)| *k == key);
    let same_page = view.pictures.iter().rev().find(|(k, _)| k.page == key.page);
    exact.or(same_page).map(|(_, px)| px.clone())
}

fn texture_id(path: &str, key: PageKey) -> egui::Id {
    egui::Id::NULL.with(("pdf-texture", path, key.page, key.scale_milli))
}

/// The texture of a picture, uploaded on first use.
pub fn texture(ctx: &egui::Context, path: &str, key: PageKey, px: &Pixels) -> TextureHandle {
    let id = texture_id(path, key);
    if let Some(t) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return t;
    }
    let image =
        ColorImage::from_rgba_unmultiplied([px.width as usize, px.height as usize], &px.rgba);
    let handle = ctx.load_texture(
        format!("pdf-{}-{}", key.page, key.scale_milli),
        image,
        TextureOptions::LINEAR,
    );
    let list_id = egui::Id::NULL.with("pdf-texture-list");
    ctx.data_mut(|d| {
        d.insert_temp(id, handle.clone());
        let mut list: Vec<egui::Id> = d.get_temp(list_id).unwrap_or_default();
        list.push(id);
        d.insert_temp(list_id, list);
    });
    handle
}

/// Let go of textures whose picture the state no longer holds.
fn prune(ctx: &egui::Context, view: &PdfView) {
    let live: Vec<egui::Id> = view
        .pictures
        .iter()
        .map(|(k, _)| texture_id(&view.path, *k))
        .collect();
    let list_id = egui::Id::NULL.with("pdf-texture-list");
    ctx.data_mut(|d| {
        let list: Vec<egui::Id> = d.get_temp(list_id).unwrap_or_default();
        let (keep, drop): (Vec<_>, Vec<_>) = list.into_iter().partition(|id| live.contains(id));
        drop.iter().for_each(|id| d.remove::<TextureHandle>(*id));
        d.insert_temp(list_id, keep);
    });
}

/// The scale (thousandths of a pixel per point) and the size on screen in logical pixels.
fn layout(view: &PdfView, avail: Vec2, ppp: f32, page: (f32, f32)) -> (u32, f32) {
    let wide = (avail.x - MARGIN).max(100.0) / page.0.max(1.0);
    let high = (avail.y - MARGIN).max(100.0) / page.1.max(1.0);
    let logical = match view.zoom {
        Zoom::FitWidth => wide,
        Zoom::FitPage => wide.min(high),
        Zoom::Percent(p) => p as f32 / 100.0 * POINT_PX,
    };
    let milli = ((logical * ppp * 20.0).round() * 50.0).clamp(50.0, 16000.0);
    (milli as u32, logical)
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

fn paint_words(view: &PdfView, ui: &egui::Ui, rect: Rect, logical: f32) {
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
            paint_words(view, ui, rect, logical);
            select(view, &response, rect, logical, out);
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
