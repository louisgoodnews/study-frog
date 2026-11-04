//! Live preview rendering of the PDF document's first pages.

use eframe::egui::{self, ColorImage, Frame, RichText, TextureOptions};

use crate::pdf_export_types::PdfExportState;
use crate::theme::palette_of;

/// Draw the preview card showing the first pages and layout summary.
pub fn draw_preview(form: &PdfExportState, ui: &mut egui::Ui) {
    let p = palette_of(ui);
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new("Document Preview").strong());
            if form.previewing {
                ui.weak("Updating…");
            } else if let Some(prev) = &form.preview {
                ui.label(
                    RichText::new(format!(
                        "{} page{}, {} item{}",
                        prev.total_pages,
                        if prev.total_pages == 1 { "" } else { "s" },
                        prev.item_count,
                        if prev.item_count == 1 { "" } else { "s" }
                    ))
                    .color(p.text_weak),
                );
            }
        });

        if let Some(err) = &form.error {
            ui.label(RichText::new(err).color(ui.visuals().error_fg_color));
            return;
        }

        let Some(prev) = &form.preview else {
            if !form.previewing {
                ui.weak("No preview available yet.");
            }
            return;
        };

        if prev.pages.is_empty() {
            ui.weak("This export contains no printable pages.");
            return;
        }

        ui.horizontal_top(|ui| {
            for page in &prev.pages {
                draw_page_box(ui, page);
            }
        });
    });
}

fn draw_page_box(ui: &mut egui::Ui, page: &crate::pdf_export_types::PreviewPage) {
    let p = palette_of(ui);
    Frame::canvas(ui.style())
        .fill(p.surface)
        .stroke(egui::Stroke::new(1.0, p.border))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(170.0);
            ui.set_min_height(220.0);
            ui.label(
                RichText::new(format!("Page {}", page.page_number))
                    .small()
                    .color(p.text_weak),
            );

            if let Some(px) = &page.pixels {
                let img = ColorImage::from_rgba_unmultiplied(
                    [px.width as usize, px.height as usize],
                    &px.rgba,
                );
                let tex = ui.ctx().load_texture(
                    format!("pdf-export-thumb-{}", page.page_number),
                    img,
                    TextureOptions::LINEAR,
                );
                let aspect = px.width as f32 / px.height.max(1) as f32;
                let w = 154.0;
                let h = w / aspect;
                ui.image((tex.id(), egui::vec2(w, h.min(180.0))));
            } else {
                for heading in &page.headings {
                    ui.label(RichText::new(heading).strong());
                }
                if !page.snippet.is_empty() {
                    ui.label(RichText::new(&page.snippet).small());
                }
            }
        });
}
