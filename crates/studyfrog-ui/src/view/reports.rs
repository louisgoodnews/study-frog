//! The reports: retention by time since the last review, how long answers take, and the review
//! load ahead. Every bar carries its numbers as text, so nothing depends on seeing the bar.

use eframe::egui::{self, ProgressBar};
use studyfrog_core::event::{Difficulty, difficulty_name};
use studyfrog_report::latency::{EDGES_MS, LatencyStats};
use studyfrog_report::retention::{RetentionBucket, retention_rate};
use studyfrog_service::reports::{BacktestReport, ReportBundle};

use crate::explorer_ui::{ExplorerMsg, ReportTab};
use crate::msg::Msg;
use crate::state::State;

fn seconds(ms: u32) -> String {
    format!("{:.1} s", f64::from(ms) / 1000.0)
}

fn bucket_label(b: &RetentionBucket) -> String {
    match (b.from_days, b.to_days) {
        (f, t) if f == t => format!("{f} day{}", if f == 1 { "" } else { "s" }),
        (f, u32::MAX) => format!("{f}+ days"),
        (f, t) => format!("{f}-{t} days"),
    }
}

fn bin_label(i: usize) -> String {
    let s = |ms: u32| ms / 1000;
    match i {
        0 => format!("under {} s", s(EDGES_MS[0])),
        i if i == EDGES_MS.len() => format!("{} s or more", s(EDGES_MS[i - 1])),
        i => format!("{}-{} s", s(EDGES_MS[i - 1]), s(EDGES_MS[i])),
    }
}

fn day_label(i: usize) -> String {
    match i {
        0 => "Today".to_owned(),
        1 => "Tomorrow".to_owned(),
        i => format!("In {i} days"),
    }
}

fn bar(ui: &mut egui::Ui, label: &str, fraction: f32, text: String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            ProgressBar::new(fraction.clamp(0.0, 1.0))
                .desired_width(240.0)
                .text(text),
        );
    });
}

/// The tabs, then either the performance explorer or the three memory reports.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.heading("Reports");
    ui.horizontal(|ui| {
        for (tab, label) in [
            (ReportTab::Performance, "Performance"),
            (ReportTab::Memory, "Memory and forecast"),
        ] {
            if ui
                .selectable_label(state.explorer.tab == tab, label)
                .clicked()
            {
                out.push(Msg::Explorer(ExplorerMsg::Tab(tab)));
            }
        }
    });
    match state.explorer.tab {
        ReportTab::Performance => crate::view::explorer::draw(state, ui, out),
        ReportTab::Memory => memory(state, ui),
    }
}

/// All three memory reports.
fn memory(state: &State, ui: &mut egui::Ui) {
    let Some(r) = &state.reports else {
        ui.label("Computing the reports…");
        return;
    };
    if r.reviews == 0 {
        ui.label("There are no reviews yet. Rehearse a few items and come back.");
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        retention(r, ui);
        latency(r, ui);
        forecast(r, ui);
    });
}

fn retention(r: &ReportBundle, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.heading("Retention");
    ui.weak("How often an item was recalled, by the time since its previous review.");
    for b in r.retention.iter().filter(|b| b.reviews > 0) {
        let rate = retention_rate(b).unwrap_or(0.0);
        bar(
            ui,
            &bucket_label(b),
            rate,
            format!("{:.0} % of {}", rate * 100.0, b.reviews),
        );
    }
}

fn stats_line(s: &LatencyStats) -> String {
    format!(
        "median {}, 90 % under {}, 95 % under {} ({} answers)",
        seconds(s.p50),
        seconds(s.p90),
        seconds(s.p95),
        s.n
    )
}

fn latency(r: &ReportBundle, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.heading("Time to answer");
    let Some(stats) = &r.latency else {
        ui.label("No answer times recorded yet.");
        return;
    };
    ui.label(stats_line(stats));
    let most = r.histogram.iter().copied().max().unwrap_or(1).max(1);
    for (i, n) in r.histogram.iter().enumerate().filter(|(_, n)| **n > 0) {
        bar(ui, &bin_label(i), *n as f32 / most as f32, n.to_string());
    }
    for m in &r.by_mark {
        let name = m
            .mark
            .map_or("not marked", |d: Difficulty| difficulty_name(d));
        ui.label(format!("Marked {name}: {}", stats_line(&m.stats)));
    }
}

fn backtest_line(b: &BacktestReport) -> String {
    let Some(e) = &b.error else {
        return "The forecast could not be checked yet.".to_owned();
    };
    let side = if e.bias >= 0.0 { "high" } else { "low" };
    let basis = if b.simulated {
        "a simulated history (yours is too short)"
    } else {
        "your own history"
    };
    format!(
        "Checked on {basis}: weekly totals were off by {:.0} %, and the forecast ran {:.0} % {side} overall.",
        e.weekly_error * 100.0,
        e.bias.abs() * 100.0
    )
}

fn forecast(r: &ReportBundle, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.heading("Reviews ahead");
    let total: f32 = r.forecast.iter().sum();
    ui.label(format!(
        "About {total:.0} reviews in the next {} days.",
        r.forecast.len()
    ));
    let most = r.forecast.iter().copied().fold(1.0_f32, f32::max);
    for (i, n) in r.forecast.iter().take(7).enumerate() {
        bar(ui, &day_label(i), n / most, format!("{n:.0}"));
    }
    for (week, chunk) in r.forecast.chunks(7).enumerate().skip(1) {
        let week_total: f32 = chunk.iter().sum();
        ui.label(format!(
            "Days {}-{}: about {week_total:.0}",
            week * 7,
            week * 7 + chunk.len() - 1
        ));
    }
    ui.weak(backtest_line(&r.backtest));
}
