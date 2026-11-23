//! The Cognitive profile tab (STORY-0178, BUG-0030): the learning velocity curve, the answer-time
//! distribution, the semantic accuracy trend, the metacognitive calibration and the timing, recall
//! and consolidation charts, the estimated difficulty of each item and the adaptive schedule against the
//! fixed one (EPIC-060, STORY-0177). Read-only; the numbers come from the review history and the telemetry.

use eframe::egui::{self, ProgressBar, RichText};
use studyfrog_service::cognitive_items::ItemDifficulty;
use studyfrog_service::cognitive_report::{Calibration, CognitiveProfile};
use studyfrog_service::schedule_compare::ScheduleCompare;

use crate::msg::Msg;
use crate::state::State;
use crate::view::reports_cognitive_charts::{bars, curve, latency_scatter};

/// The gap, in shares, beyond which the calibration note names a mismatch.
const GAP: f32 = 0.15;
/// Width of one answer-time bin, in seconds, as the chart labels it.
const BIN_S: u32 = 2;
/// The lowest and highest values on the difficulty scale.
const SCALE: (f64, f64) = (1.0, 10.0);

/// Draw the cognitive profile tab.
pub fn draw(state: &State, ui: &mut egui::Ui, _out: &mut Vec<Msg>) {
    let Some(reports) = &state.reports else {
        ui.label("Computing the reports…");
        return;
    };
    let p = &reports.cognitive;
    egui::ScrollArea::vertical().show(ui, |ui| {
        crate::view::kit::column(ui, crate::view::kit::READING_WIDTH, |ui| {
            ui.label(format!(
                "{} reviews in the history, {} with answer timing.",
                p.answers, p.timed
            ));
            ui.add_space(12.0);
            calibration(ui, p.calibration);
            ui.add_space(16.0);
            velocity(ui, p);
            ui.add_space(16.0);
            answer_times(ui, p);
            ui.add_space(16.0);
            semantic(ui, p);
            ui.add_space(16.0);
            section(ui, "Answer time against the length of the words");
            if p.length_points.is_empty() {
                ui.label("No timed answers yet.");
            } else {
                latency_scatter(ui, &p.length_points);
            }
            ui.add_space(16.0);
            section(ui, "Share of answers that passed, cumulative by week");
            if p.weekly_pass.is_empty() {
                ui.label("No reviews yet.");
            } else {
                curve(
                    ui,
                    &p.weekly_pass,
                    1.0,
                    "100%",
                    &end_label(&p.weekly_pass, "week"),
                );
            }
            ui.add_space(16.0);
            consolidation(ui, p);
            ui.add_space(16.0);
            difficulty_list(ui, &p.item_difficulty);
            ui.add_space(16.0);
            schedule(ui, p.schedule);
        });
    });
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).strong());
}

fn bar(ui: &mut egui::Ui, label: &str, share: f32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            ProgressBar::new(share.clamp(0.0, 1.0))
                .desired_width(200.0)
                .text(format!("{:.0}%", share * 100.0)),
        );
    });
}

/// Ratings against timing, and what the gap between them says.
fn calibration(ui: &mut egui::Ui, c: Option<Calibration>) {
    section(ui, "Metacognitive calibration");
    let Some(c) = c else {
        ui.label("Answer times are needed to compare your ratings with your timing.");
        return;
    };
    bar(ui, "Rated Easy", c.easy_share);
    bar(ui, "Answered in time", c.in_time_share);
    let gap = c.easy_share - c.in_time_share;
    let note = if gap > GAP {
        "You rate answers Easy more often than you answer them in time. The ratings may be generous."
    } else if gap < -GAP {
        "You answer in time more often than you rate them Easy. You may be under-rating your recall."
    } else {
        "Your ratings and your timing agree closely."
    };
    ui.label(note);
}

/// The label under the last point of a curve: the unit and the last x, for example `week 3`.
fn end_label(points: &[(f32, f32)], unit: &str) -> String {
    let last = points.last().map_or(0.0, |(x, _)| *x);
    format!("{unit} {last:.0}")
}

/// Learning velocity: the figure, and the curve of passing answers per minute of answering, by
/// cumulative answering minutes.
fn velocity(ui: &mut egui::Ui, p: &CognitiveProfile) {
    section(ui, "Learning velocity");
    match p.velocity {
        Some(v) => ui.label(format!("{v:.2} passing answers per minute of answering.")),
        None => ui.label("Needs more timed answers to measure."),
    };
    if p.velocity_curve.is_empty() {
        return;
    }
    let top = p
        .velocity_curve
        .iter()
        .fold(f32::EPSILON, |m, (_, y)| m.max(*y));
    let minutes = p.velocity_curve.last().map_or(0.0, |(x, _)| *x);
    curve(
        ui,
        &p.velocity_curve,
        top,
        &format!("{top:.2} per min"),
        &format!("{minutes:.1} min answering"),
    );
}

/// The distribution of answer times.
fn answer_times(ui: &mut egui::Ui, p: &CognitiveProfile) {
    section(ui, "Answer times, in bins of 2 seconds");
    if p.latency_bins.iter().all(|c| *c == 0) {
        ui.label("No timed answers yet.");
    } else {
        bars(ui, &p.latency_bins, BIN_S);
    }
}

/// The semantic accuracy trend: the running mean of the judged answers' scores, or why there is none.
fn semantic(ui: &mut egui::Ui, p: &CognitiveProfile) {
    section(ui, "Semantic accuracy over time");
    if p.semantic_trend.is_empty() {
        ui.label(
            "Not shown yet: no answer has a semantic score. A typed answer that is judged records one; flip-mode answers have nothing to score.",
        );
        return;
    }
    ui.label(format!("{} judged answers.", p.scored));
    let n = p.semantic_trend.last().map_or(0.0, |(x, _)| *x);
    curve(
        ui,
        &p.semantic_trend,
        1.0,
        "100%",
        &format!("answer {n:.0}"),
    );
}

/// The mean days between passing reviews, by week.
fn consolidation(ui: &mut egui::Ui, p: &CognitiveProfile) {
    section(ui, "Days between passing reviews, by week (consolidation)");
    if p.consolidation.is_empty() {
        ui.label("No passing reviews with a gap yet.");
        return;
    }
    let top = p.consolidation.iter().fold(1.0_f32, |m, (_, d)| m.max(*d));
    curve(
        ui,
        &p.consolidation,
        top,
        &format!("{top:.0} days"),
        &end_label(&p.consolidation, "week"),
    );
}

/// The items by estimated difficulty, hardest first, each with its bar on the 1 to 10 scale.
pub fn difficulty_list(ui: &mut egui::Ui, items: &[ItemDifficulty]) {
    ui.label(RichText::new("Estimated difficulty by item (1 easy to 10 hard)").strong());
    if items.is_empty() {
        ui.label("No item has timed answers yet. Each timed rating records one.");
        return;
    }
    for it in items {
        difficulty_row(ui, it);
    }
}

fn difficulty_row(ui: &mut egui::Ui, it: &ItemDifficulty) {
    let title = if it.title.is_empty() {
        "(item no longer indexed)"
    } else {
        it.title.as_str()
    };
    let place = if it.folder.is_empty() {
        String::new()
    } else {
        format!(" in {}", it.folder)
    };
    ui.label(format!(
        "{title}{place}: from {} answer{}",
        it.answers,
        if it.answers == 1 { "" } else { "s" }
    ));
    let share = ((it.difficulty - SCALE.0) / (SCALE.1 - SCALE.0)).clamp(0.0, 1.0) as f32;
    ui.add(
        ProgressBar::new(share)
            .desired_width(200.0)
            .text(format!("{:.1}", it.difficulty)),
    );
}

/// The intervals after the second review, with the plain prior against the calibrated start.
pub fn schedule(ui: &mut egui::Ui, c: Option<ScheduleCompare>) {
    ui.label(RichText::new("Adaptive schedule against the fixed one (second review)").strong());
    let Some(c) = c.filter(|c| c.items > 0) else {
        ui.label("No item has a second review yet, so there is nothing to compare.");
        return;
    };
    ui.label(format!(
        "{} items. Interval: fixed prior {:.1} days, calibrated start {:.1} days ({} of {} differ).",
        c.items, c.fixed_days, c.calibrated_days, c.differ, c.items
    ));
    ui.label(format!(
        "Stability: fixed prior {:.2} days, calibrated start {:.2} days.",
        c.fixed_stability, c.calibrated_stability
    ));
    ui.label(
        "The first review is the same under both, since its stability depends on the rating only. The starting difficulty shapes the later reviews.",
    );
}
