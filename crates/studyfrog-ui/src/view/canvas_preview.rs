//! Expandable card preview popovers for canvas nodes referencing vault items.
//!
//! Follows Invariant P1: zero inherent impl blocks; all logic in free functions.

use eframe::egui::{self, RichText};
use studyfrog_md::ulid::parse_ulid;

use crate::canvas_types::{CanvasMsg, CardPreviewState};
use crate::msg::Msg;

/// Renders an expandable floating card preview popover for an active node.
pub fn draw_preview(ctx: &egui::Context, preview: &CardPreviewState, out: &mut Vec<Msg>) {
    let mut is_open = true;
    egui::Window::new("Card Preview")
        .open(&mut is_open)
        .resizable(true)
        .collapsible(false)
        .default_size([340.0, 220.0])
        .min_size([260.0, 160.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let heading_size = crate::typo::size(ui, crate::typo::Tier::Heading);
                ui.label(RichText::new(&preview.title).strong().size(heading_size));
            });
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("ID: {}", preview.item_id))
                        .small()
                        .weak(),
                );
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(240.0)
                .show(ui, |ui| {
                    ui.label(&preview.content);
                });

            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                if let Some(uid) = parse_ulid(&preview.item_id)
                    && ui
                        .button("Open in Editor")
                        .on_hover_text("Opens this in the Markdown editor.")
                        .clicked()
                {
                    out.push(Msg::OpenItem(uid));
                    out.push(Msg::Canvas(CanvasMsg::ClosePreview));
                }
                if ui.button("Close").on_hover_text("Closes this.").clicked() {
                    out.push(Msg::Canvas(CanvasMsg::ClosePreview));
                }
            });
        });

    if !is_open {
        out.push(Msg::Canvas(CanvasMsg::ClosePreview));
    }
}
