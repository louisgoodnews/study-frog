//! A gradient panel for a page's headline (EPIC-070, STORY-0215): the panel reserves its full height in the layout,
//! the gradient is painted behind it, and the content gets the padding and the text colour the gradient needs. Shared
//! by Home and the stack page.

use eframe::egui::{self, Shape};

use crate::theme::PAD_CARD;
use crate::theme::palette_of;
use crate::theme::style::{Style, style_of};
use crate::view::home_ring;

/// A panel `height` tall across the available width, with the gradient behind `add`, which gets the style of the
/// panel for its text and buttons.
pub(super) fn panel(ui: &mut egui::Ui, height: f32, add: impl FnOnce(&mut egui::Ui, Style)) {
    let s = style_of(&palette_of(ui));
    let width = ui.available_width();
    ui.allocate_ui(egui::vec2(width, height), |ui| {
        ui.painter().add(Shape::mesh(home_ring::gradient_mesh(
            ui.max_rect(),
            s.gradient[0],
            s.gradient[1],
        )));
        egui::Frame::new()
            .inner_margin(PAD_CARD)
            .show(ui, |ui| add(ui, s));
    });
}
