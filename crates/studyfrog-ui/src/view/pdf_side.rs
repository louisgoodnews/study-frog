//! The side panel of the PDF screen: the outline, or a thumbnail of every page.

use eframe::egui::{self, ScrollArea, Sense, Vec2};

use studyfrog_store::overview::StackRow;

use crate::msg::Msg;
use crate::pdf_types::{PageKey, PdfMsg, PdfView, Side, THUMB_SCALE_MILLI};
use crate::view::pdf_page::{picture_for, texture};
use crate::view::{pdf_extract, pdf_notes};

const THUMB_WIDTH: f32 = 120.0;

fn outline(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(info) = &view.info else { return };
    if info.outline.is_empty() {
        ui.weak("This PDF has no outline.");
        return;
    }
    for entry in &info.outline {
        ui.horizontal(|ui| {
            ui.add_space(entry.level as f32 * 12.0);
            let here = entry.page == Some(view.page);
            let button = egui::Button::selectable(here, &entry.title);
            if ui.add(button).clicked()
                && let Some(page) = entry.page
            {
                out.push(Msg::Pdf(PdfMsg::GoTo(page)));
            }
        });
    }
}

fn thumbnail(view: &PdfView, page: usize, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let size = view
        .info
        .as_ref()
        .and_then(|i| i.pages.get(page))
        .map_or((612.0, 792.0), |s| (s.width, s.height));
    let height = THUMB_WIDTH * size.1 / size.0.max(1.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(THUMB_WIDTH, height), Sense::click());
    let key = PageKey {
        page,
        scale_milli: THUMB_SCALE_MILLI,
    };
    if ui.is_rect_visible(rect) {
        match picture_for(view, key) {
            Some(px) => {
                let tex = texture(ui.ctx(), &view.path, key, &px);
                egui::Image::new(&tex).paint_at(ui, rect);
            }
            None => {
                ui.painter()
                    .rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
                out.push(Msg::Pdf(PdfMsg::Need(key)));
            }
        }
    }
    if page == view.page {
        ui.painter().rect_stroke(
            rect,
            2.0,
            ui.visuals().selection.stroke,
            egui::StrokeKind::Outside,
        );
    }
    ui.label(format!("{}", page + 1));
    if response
        .on_hover_text(format!("Page {}", page + 1))
        .clicked()
    {
        out.push(Msg::Pdf(PdfMsg::GoTo(page)));
    }
}

/// Draw the panel that `view.side` names.
pub fn draw(view: &PdfView, stacks: &[StackRow], ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ScrollArea::vertical()
        .id_salt("pdf-side-scroll")
        .show(ui, |ui| match view.side {
            Side::Outline => outline(view, ui, out),
            Side::Pages => {
                let pages = view.info.as_ref().map_or(0, |i| i.page_count);
                for page in 0..pages {
                    thumbnail(view, page, ui, out);
                }
            }
            Side::Notes => pdf_notes::list(view, ui, out),
            Side::Extract => pdf_extract::draw(view, stacks, ui, out),
            Side::Closed => {}
        });
}
