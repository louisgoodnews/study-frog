//! The Prompts headline (EPIC-073, STORY-0238): the title on the gradient panel, then the prompts.

use eframe::egui::{self, RichText};

use crate::view::gradient_panel;

/// The height of the headline panel, in points.
const HEADER_HEIGHT: f32 = 64.0;

/// The title, on the gradient panel.
pub fn headline(ui: &mut egui::Ui) {
    gradient_panel::panel(ui, HEADER_HEIGHT, |ui, s| {
        ui.label(
            RichText::new("Prompts")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong()
                .color(s.gradient_text),
        );
    });
}
