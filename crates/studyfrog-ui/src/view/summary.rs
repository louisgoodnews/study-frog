//! The end-of-run screen: the counts as tiles and bars, and a scrollable card for every item.

use eframe::egui::{self, RichText};
use studyfrog_core::event::{Difficulty, difficulty_name};
use studyfrog_core::plan::ScheduleEffect;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunState;
use studyfrog_slm::score::Verdict;

use crate::msg::Msg;
use crate::review::review_rows;
use crate::run_state::RunView;
use crate::summary::summarize;
use crate::theme::palette_of;
use crate::view::{kit, summary_items};

pub(crate) fn rating_name(r: Rating) -> &'static str {
    match r {
        Rating::Again => "Again",
        Rating::Hard => "Hard",
        Rating::Good => "Good",
        Rating::Easy => "Easy",
    }
}

pub(crate) fn mark_name(d: Option<Difficulty>) -> String {
    d.map_or_else(|| "-".to_owned(), |d| difficulty_name(d).to_owned())
}

pub(crate) fn verdict_label(v: Verdict) -> &'static str {
    match v {
        Verdict::Correct => "Correct",
        Verdict::Partial => "Partly right",
        Verdict::Incorrect => "Not quite",
    }
}

pub(crate) fn rating_text(r: Option<Rating>) -> &'static str {
    r.map_or("-", rating_name)
}

/// Heading and the way back on top, the counts, then one card per item in a scrolling list.
pub fn draw(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let stopped = run.session.run.state == RunState::Aborted;
    ui.horizontal(|ui| {
        let title = if stopped {
            "Run stopped"
        } else {
            "Run finished"
        };
        ui.label(RichText::new(title).size(26.0).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::primary_button(ui, "Back to setup").clicked() {
                out.push(Msg::LeaveRun);
            }
        });
    });
    counts(run, ui);
    ui.weak(crate::run_time::run_line(
        run.session.run.plan.pacing,
        studyfrog_rehearsal::pacing::run_active_ms(&run.events),
        summarize(&run.events, &run.latencies)
            .rated
            .iter()
            .sum::<usize>() as u32,
    ));
    ui.weak(crate::run_time::run_line(
        run.session.run.plan.pacing,
        studyfrog_rehearsal::pacing::run_active_ms(&run.events),
        summarize(&run.events, &run.latencies)
            .rated
            .iter()
            .sum::<usize>() as u32,
    ));
    let updates = run.session.run.plan.effect == ScheduleEffect::UpdateSchedule;
    if !updates {
        ui.weak("Practice only: your schedule was not changed.");
    }
    let rows = review_rows(run);
    if rows.is_empty() {
        return;
    }
    ui.add_space(8.0);
    kit::section(ui, "Review");
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for row in &rows {
                summary_items::card(run, row, updates, ui, out);
            }
        });
}

fn counts(run: &RunView, ui: &mut egui::Ui) {
    let s = summarize(&run.events, &run.latencies);
    let rated: usize = s.rated.iter().sum();
    if rated + s.skipped == 0 {
        ui.label("There was nothing to rehearse.");
        return;
    }
    let p = palette_of(ui);
    let colors = [p.danger, p.warning, p.success, p.accent];
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            kit::stat(ui, &rated.to_string(), "rated");
            ui.add_space(18.0);
            kit::stat(ui, &s.skipped.to_string(), "skipped");
            ui.add_space(18.0);
            kit::stat(
                ui,
                &format!("{:.0} s", f64::from(s.total_ms) / 1000.0),
                "in total",
            );
            if let Some(ms) = s.median_latency_ms {
                ui.add_space(18.0);
                kit::stat(
                    ui,
                    &format!("{:.1} s", f64::from(ms) / 1000.0),
                    "median time to answer",
                );
            }
        });
        ui.add_space(6.0);
        for (i, name) in ["Again", "Hard", "Good", "Easy"].into_iter().enumerate() {
            let share = if rated == 0 {
                0.0
            } else {
                s.rated[i] as f32 / rated as f32
            };
            ui.horizontal(|ui| {
                ui.colored_label(colors[i], format!("{name}: {}", s.rated[i]));
                ui.add(
                    egui::ProgressBar::new(share)
                        .fill(colors[i])
                        .desired_height(8.0),
                );
            });
        }
    });
}
