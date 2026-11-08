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
        RunMode::Exam => "Exam",
    }
}

/// An exam's result, or why it has none.
pub(crate) fn exam_text(r: &RunSummary) -> Option<String> {
    if r.mode != RunMode::Exam {
        return None;
    }
    Some(match &r.exam {
        Some(e) => format!(
            "{:.1} % ({}), {}, {} of {} answered",
            e.percent,
            studyfrog_core::exam::grade_badge(e.grade),
            if e.passed { "passed" } else { "not passed" },
            e.answered,
            e.items
        ),
        None => "not graded".to_owned(),
    })
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
    let counts =
        exam_text(r).unwrap_or_else(|| format!("{} rated, {} skipped", r.rated, r.skipped));
    format!(
        "{date}: {}, {counts}, {how}; {}",
        mode_text(r.mode),
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
            (Some(RunMode::Exam), "Exam"),
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
        crate::view::kit::column(ui, crate::view::kit::READING_WIDTH, |ui| {
            for r in shown {
                ui.label(line(r));
            }
        });
    });
}
