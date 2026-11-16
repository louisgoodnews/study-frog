//! FSRS parameter calibration section in Rehearsal settings.
//!
//! Provides the 'Optimize FSRS Parameters' button and displays evaluation
//! metrics (initial loss, final loss, improvement %, and RMSE).

use std::path::Path;

use eframe::egui::{self, Grid, RichText};
use rusqlite::{Connection, OpenFlags};
use studyfrog_service::fsrs_optimize::{
    FsrsOptimizationMetrics, load_review_records, run_fsrs_optimization,
};

use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;

/// Draw the FSRS parameter optimization section on the Rehearsal settings tab.
pub fn draw(state: &State, ui: &mut egui::Ui, _out: &mut Vec<Msg>) {
    crate::view::kit::section(ui, "FSRS parameter optimization");
    let pal = palette_of(ui);

    ui.label(
        RichText::new(
            "Fit personal FSRS-6 weights to your historical review logs to improve recall accuracy.",
        )
        .color(pal.text_weak),
    );

    let id = egui::Id::NULL.with("fsrs_optimization_metrics");
    let mut metrics: Option<FsrsOptimizationMetrics> = ui.data(|d| d.get_temp(id));

    let btn = egui::Button::new("Optimize FSRS Parameters");
    if ui.add(btn).clicked()
        && let Some(vault) = &state.vault
    {
        let db_path = Path::new(&vault.root)
            .join(".studyfrog")
            .join("index.sqlite");
        if let Ok(conn) = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            && let Ok(samples) = load_review_records(&conn)
            && let Some(res) = run_fsrs_optimization(&samples, None)
        {
            ui.data_mut(|d| d.insert_temp(id, res));
            metrics = Some(res);
        }
    }

    if let Some(m) = metrics {
        ui.add_space(8.0);
        draw_metrics(ui, &m);
    }
}

/// Render the formatted evaluation metrics in a grid.
pub fn draw_metrics(ui: &mut egui::Ui, metrics: &FsrsOptimizationMetrics) {
    let pal = palette_of(ui);
    Grid::new("fsrs_metrics_table")
        .num_columns(2)
        .spacing([32.0, 6.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Initial loss").color(pal.text_weak));
            ui.label(format!("{:.4}", metrics.initial_loss));
            ui.end_row();

            ui.label(RichText::new("Final loss").color(pal.text_weak));
            ui.label(format!("{:.4}", metrics.final_loss));
            ui.end_row();

            ui.label(RichText::new("Improvement").color(pal.text_weak));
            ui.label(
                RichText::new(format!("{:.2} %", metrics.loss_reduction_pct)).color(pal.accent),
            );
            ui.end_row();

            ui.label(RichText::new("RMSE").color(pal.text_weak));
            ui.label(format!("{:.4}", metrics.rmse));
            ui.end_row();

            ui.label(RichText::new("Reviews evaluated").color(pal.text_weak));
            ui.label(format!("{}", metrics.sample_count));
            ui.end_row();
        });
}

/// Set metrics in egui temporary storage (useful for testing or external state).
pub fn set_metrics(ctx: &egui::Context, metrics: FsrsOptimizationMetrics) {
    let id = egui::Id::NULL.with("fsrs_optimization_metrics");
    ctx.data_mut(|d| d.insert_temp(id, metrics));
}
