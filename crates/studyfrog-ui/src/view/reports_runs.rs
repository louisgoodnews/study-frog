//! The history of runs: mode, pacing, planned and used time and answers a minute, with a filter by mode.

use eframe::egui;
use studyfrog_core::civil::format_date;
use studyfrog_core::run::RunMode;
use studyfrog_service::run_history::RunSummary;

use crate::explorer_ui::ExplorerMsg;
use crate::msg::Msg;
use crate::run_time::run_line;
use crate::state::State;

fn mode_text(m: RunMode) -> &'static str {
    match m {
        RunMode::Flip => "Flip cards",
        RunMode::Answer => "Type answers",
    }
}

fn line(r: &RunSummary) -> String {
    let date = format_date(studyfrog_core::civil::date_from_days(
        r.started_ms.div_euclid(86_400_000),
    ));
    let how = if r.state == "finished" {
        "finished"
    } else {
        "stopped early"
    };
    format!(
        "{date}: {}, {} rated, {} skipped, {how}; {}",
        mode_text(r.mode),
        r.rated,
        r.skipped,
        run_line(r.pacing, r.used_ms, r.rated)
    )
}

/// The tab.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(b) = &state.reports else {
        ui.label("Computing the reports…");
        return;
    };
    let mode = state.explorer.mode;
    ui.horizontal(|ui| {
        ui.label("Show");
        for (m, label) in [
            (None, "All runs"),
            (Some(RunMode::Flip), "Flip cards"),
            (Some(RunMode::Answer), "Type answers"),
        ] {
            if ui.selectable_label(mode == m, label).clicked() {
                out.push(Msg::Explorer(ExplorerMsg::Mode(m)));
            }
        }
    });
    let shown: Vec<&RunSummary> = b
        .runs
        .iter()
        .filter(|r| mode.is_none_or(|m| r.mode == m))
        .collect();
    if shown.is_empty() {
        ui.weak("No runs to show.");
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        for r in shown {
            ui.label(line(r));
        }
    });
}
