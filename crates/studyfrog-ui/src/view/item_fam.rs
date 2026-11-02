//! The familiarity card of the item page: the value now, what drove it and how it moved.

use eframe::egui::{self, Stroke, vec2};
use studyfrog_core::civil::{date_from_days, format_date};

use crate::editor::Editor;
use crate::theme::palette_of;
use crate::view::kit;

/// The card; nothing for an item with no history.
pub fn card(e: &Editor, ui: &mut egui::Ui) {
    let Some(curve) = e.curve.as_ref().filter(|c| !c.points.is_empty()) else {
        return;
    };
    let now = curve.points.last().map_or(0, |p| p.percent);
    kit::section(ui, "Familiarity");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(format!("Familiarity {now} %")).strong());
        if !curve.drivers.is_empty() {
            ui.weak(&curve.drivers);
        }
        let first = curve.points.first().map_or(0, |p| p.day);
        let last = curve.points.last().map_or(0, |p| p.day);
        let (rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), 48.0), egui::Sense::hover());
        let span = (last - first).max(1) as f32;
        let line: Vec<egui::Pos2> = curve
            .points
            .iter()
            .map(|p| {
                let x = rect.left() + rect.width() * (p.day - first) as f32 / span;
                egui::pos2(
                    x,
                    rect.bottom() - rect.height() * f32::from(p.percent) / 100.0,
                )
            })
            .collect();
        ui.painter().add(egui::Shape::line(
            line,
            Stroke::new(2.0, palette_of(ui).accent),
        ));
        let summary: Vec<String> = curve
            .points
            .iter()
            .rev()
            .take(6)
            .rev()
            .map(|p| format!("{} {} %", format_date(date_from_days(p.day)), p.percent))
            .collect();
        ui.weak(format!("Over time: {}", summary.join(", ")));
    });
    ui.add_space(10.0);
}
