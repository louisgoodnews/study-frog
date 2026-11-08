//! Rows of a name, a bar with its main number, and the remaining numbers as text beside it: for reports whose
//! numbers do not fit on a bar. Nothing depends on seeing the bar.

use eframe::egui::{self, RichText};

/// One row.
pub struct DetailRow {
    /// The name in the first column.
    pub label: String,
    /// How full the bar is, 0 to 1.
    pub fraction: f32,
    /// The main number, written on the bar.
    pub value: String,
    /// The other numbers, written after the bar.
    pub detail: String,
}

/// The rows as a grid.
pub fn detail_rows(ui: &mut egui::Ui, id: &str, rows: &[DetailRow]) {
    egui::Grid::new(id)
        .num_columns(3)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            for r in rows {
                ui.label(&r.label);
                ui.add(
                    egui::ProgressBar::new(r.fraction.clamp(0.0, 1.0))
                        .desired_width(120.0)
                        .desired_height(18.0)
                        .text(RichText::new(&r.value)),
                );
                ui.label(&r.detail);
                ui.end_row();
            }
        });
}
