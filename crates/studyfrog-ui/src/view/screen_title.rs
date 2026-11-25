//! The title of a settings or help screen on the gradient panel (EPIC-073, STORY-0227), as the Reports headline is.

use eframe::egui::{self, RichText};

use crate::view::gradient_panel;

/// The height of the headline panel, in points.
const HEADLINE_HEIGHT: f32 = 64.0;

/// The title of a screen, large and bold on the gradient panel.
pub(super) fn headline(ui: &mut egui::Ui, title: &str) {
    gradient_panel::panel(ui, HEADLINE_HEIGHT, |ui, s| {
        ui.label(
            RichText::new(title)
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong()
                .color(s.gradient_text),
        );
    });
}
