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
use crate::state::State;
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

pub(crate) fn shortcuts(run: &RunView, ui: &egui::Ui, out: &mut Vec<Msg>) {
    if ui.ctx().egui_wants_keyboard_input() {
        return;
    }
    let pressed = |k: egui::Key| ui.input(|i| i.key_pressed(k));
    if pressed(egui::Key::Escape) {
        let active = run
            .diary_prompt
            .as_ref()
            .is_none_or(|p| !p.skipped && !p.saved);
        if active {
            out.push(Msg::Diary(crate::diary_msg::DiaryMsg::PromptSkip));
        } else {
            out.push(Msg::LeaveRun);
        }
    }
    if pressed(egui::Key::Space) {
        out.push(Msg::LeaveRun);
    }
    if pressed(egui::Key::Enter) {
        let active = run
            .diary_prompt
            .as_ref()
            .is_none_or(|p| !p.skipped && !p.saved);
        if active {
            out.push(Msg::Diary(crate::diary_msg::DiaryMsg::PromptSubmit));
        }
    }
}

/// Heading and the way back on top, the counts, then one card per item in a scrolling list.
pub fn draw(state: &State, run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    shortcuts(run, ui, out);
    let outer_h = ui.available_height();
    egui::ScrollArea::vertical()
        .id_salt("summary-outer-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let stopped = run.session.run.state == RunState::Aborted;
            ui.horizontal(|ui| {
                let title = if stopped {
                    "Run stopped"
                } else {
                    "Run finished"
                };
                ui.label(
                    RichText::new(title)
                        .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                        .strong(),
                );
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
            let updates = run.session.run.plan.effect == ScheduleEffect::UpdateSchedule;
            if !updates {
                ui.weak("Practice only: your schedule was not changed.");
            }
            let rows = review_rows(run);
            if !rows.is_empty() {
                ui.add_space(4.0);
                kit::section(ui, "Review");
                let diary_active = run
                    .diary_prompt
                    .as_ref()
                    .is_some_and(|p| !p.skipped && !p.saved);
                let reserve_bottom = if diary_active { 190.0 } else { 36.0 };
                let review_height = (outer_h - 240.0 - reserve_bottom).max(240.0);
                egui::ScrollArea::vertical()
                    .id_salt("summary-review-scroll")
                    .auto_shrink([false, true])
                    .max_height(review_height)
                    .show(ui, |ui| {
                        for row in &rows {
                            summary_items::card(run, row, updates, ui, out);
                        }
                    });
            }
            ui.add_space(4.0);
            crate::view::run_diary_prompt::draw(state, run, ui, out);
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
        let rows: Vec<kit::BarRow> = ["Again", "Hard", "Good", "Easy"]
            .into_iter()
            .enumerate()
            .map(|(i, name)| kit::BarRow {
                label: format!("{name}: {}", s.rated[i]),
                fraction: if rated == 0 {
                    0.0
                } else {
                    s.rated[i] as f32 / rated as f32
                },
                text: String::new(),
                color: Some(colors[i]),
            })
            .collect();
        kit::bar_rows(ui, "summary-ratings", &rows);
    });
}
