//! Read-only view of the effective settings.

use eframe::egui::{self, Grid};
use studyfrog_core::scheduler::scheduler_name;

use crate::state::State;

/// Vault settings (with the reason when `studyfrog.toml` was not usable) and where they come from.
pub fn draw(state: &State, ui: &mut egui::Ui) {
    ui.heading("Settings");
    let Some(vault) = &state.vault else { return };
    if let Some(e) = &vault.settings_error {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("studyfrog.toml is not used: {e}"),
        );
    }
    let s = &vault.settings;
    Grid::new("settings").num_columns(2).show(ui, |ui| {
        ui.label("Default scheduler");
        ui.label(scheduler_name(s.scheduler));
        ui.end_row();
        ui.label("Target retention (FSRS)");
        ui.label(format!("{}", s.request_retention));
        ui.end_row();
        ui.label("Maximum interval (days)");
        ui.label(s.maximum_interval_days.to_string());
        ui.end_row();
        ui.label("FSRS weights");
        ui.label(if s.weights.is_some() {
            "custom"
        } else {
            "published defaults"
        });
        ui.end_row();
    });
    ui.add_space(8.0);
    ui.weak("Edit studyfrog.toml in the vault folder, and _stack.toml next to a stack, to change these.");
}
