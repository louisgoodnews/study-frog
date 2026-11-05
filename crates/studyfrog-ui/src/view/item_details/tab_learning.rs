//! The Learning tab of the item details notebook: familiarity score, driver factors,
//! and historical familiarity curve.

use eframe::egui;

use crate::editor::Editor;
use crate::view::kit;

/// Draw the Learning tab.
pub fn draw(e: &Editor, ui: &mut egui::Ui) {
    let has_curve = e.curve.as_ref().is_some_and(|c| !c.points.is_empty());
    if has_curve {
        crate::view::item_fam::card(e, ui);
    } else {
        kit::section(ui, "Familiarity");
        kit::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.weak("No review history yet. Study this item to build a familiarity curve.");
        });
    }
}
