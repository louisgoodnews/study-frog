//! A gradient panel for a page's headline (EPIC-070, STORY-0215): the content is laid out first, then the gradient is
//! painted behind it at the size the content took, so the layout below always starts after the panel. Shared by Home,
//! the stack page and the library overview.

use eframe::egui::{self, Shape};

use crate::theme::PAD_CARD;
use crate::theme::palette_of;
use crate::theme::style::{Style, style_of};
use crate::view::home_ring;

/// A panel at least `height` tall across the available width, with the gradient behind `add`, which gets the style of
/// the panel for its text and buttons.
pub(super) fn panel(ui: &mut egui::Ui, height: f32, add: impl FnOnce(&mut egui::Ui, Style)) {
    let s = style_of(&palette_of(ui));
    let slot = ui.painter().add(Shape::Noop);
    let framed = egui::Frame::new().inner_margin(PAD_CARD).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.set_min_height(height - 2.0 * PAD_CARD);
        add(ui, s);
    });
    ui.painter().set(
        slot,
        Shape::mesh(home_ring::gradient_mesh(
            framed.response.rect,
            s.gradient[0],
            s.gradient[1],
        )),
    );
}
