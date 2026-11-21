//! The Cognitive profile tab (STORY-0178): timing against length, weekly recall, learning velocity
//! and the metacognitive calibration widget. Read-only; the numbers come from the review history.

use eframe::egui::{self, ProgressBar, RichText};
use studyfrog_service::cognitive_report::{Calibration, CognitiveProfile};

use crate::msg::Msg;
use crate::state::State;
use crate::view::reports_cognitive_charts::{latency_scatter, weekly_line};

/// The gap, in shares, beyond which the calibration note names a mismatch.
const GAP: f32 = 0.15;

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
                weekly_line(ui, &p.weekly_pass, 1.0, "100%");
            }
            ui.add_space(16.0);
            section(ui, "Days between passing reviews, by week (consolidation)");
            if p.consolidation.is_empty() {
                ui.label("No passing reviews with a gap yet.");
            } else {
                let top = p.consolidation.iter().fold(1.0_f32, |m, (_, d)| m.max(*d));
                weekly_line(ui, &p.consolidation, top, &format!("{top:.0} days"));
            }
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

/// Learning velocity, in passing answers per minute of answering.
fn velocity(ui: &mut egui::Ui, p: &CognitiveProfile) {
    section(ui, "Learning velocity");
    match p.velocity {
        Some(v) => ui.label(format!("{v:.2} passing answers per minute of answering.")),
        None => ui.label("Needs more timed answers to measure."),
    };
}
