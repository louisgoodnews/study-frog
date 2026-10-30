//! The end-of-run screen: counts, and the review of each item.

use eframe::egui::{self, Grid};
use studyfrog_core::event::{Difficulty, difficulty_name};
use studyfrog_core::plan::ScheduleEffect;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunState;
use studyfrog_rehearsal::reading::{Reason, Suggestion};
use studyfrog_slm::score::Verdict;

use crate::msg::Msg;
use crate::review::{ReviewRow, review_rows};
use crate::run_state::RunView;
use crate::summary::summarize;

fn rating_name(r: Rating) -> &'static str {
    match r {
        Rating::Again => "Again",
        Rating::Hard => "Hard",
        Rating::Good => "Good",
        Rating::Easy => "Easy",
    }
}

fn mark_name(d: Option<Difficulty>) -> String {
    d.map_or_else(|| "-".to_owned(), |d| difficulty_name(d).to_owned())
}

fn verdict_label(v: Verdict) -> &'static str {
    match v {
        Verdict::Correct => "Correct",
        Verdict::Partial => "Partly right",
        Verdict::Incorrect => "Not quite",
    }
}

fn rating_text(r: Option<Rating>) -> &'static str {
    r.map_or("-", rating_name)
}

/// Counts first, then one line per item, then the way back to the setup screen.
pub fn draw(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let stopped = run.session.run.state == RunState::Aborted;
    ui.heading(if stopped {
        "Run stopped"
    } else {
        "Run finished"
    });
    counts(run, ui);
    let updates = run.session.run.plan.effect == ScheduleEffect::UpdateSchedule;
    if !updates {
        ui.weak("Practice only: your schedule was not changed.");
    }
    review_table(run, updates, ui, out);
    if ui.button("Back to setup").clicked() {
        out.push(Msg::LeaveRun);
    }
}

fn counts(run: &RunView, ui: &mut egui::Ui) {
    let s = summarize(&run.events, &run.latencies);
    let rated: usize = s.rated.iter().sum();
    if rated + s.skipped == 0 {
        ui.label("There was nothing to rehearse.");
        return;
    }
    ui.label(format!(
        "{rated} rated, {} skipped, in {:.0} s",
        s.skipped,
        f64::from(s.total_ms) / 1000.0
    ));
    for (i, name) in ["Again", "Hard", "Good", "Easy"].into_iter().enumerate() {
        ui.label(format!("{name}: {}", s.rated[i]));
    }
    if let Some(ms) = s.median_latency_ms {
        ui.label(format!(
            "Median time to answer: {:.1} s",
            f64::from(ms) / 1000.0
        ));
    }
}

fn review_table(run: &RunView, updates: bool, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let rows = review_rows(run);
    if rows.is_empty() {
        return;
    }
    ui.add_space(8.0);
    ui.heading("Review");
    let scored = rows.iter().any(|r| r.answer_match.is_some());
    Grid::new("review")
        .striped(true)
        .num_columns(5 + usize::from(scored))
        .show(ui, |ui| {
            let heads = ["Item", "Previous rating", "Your mark", "Your rating"];
            heads.iter().for_each(|h| drop(ui.strong(*h)));
            if scored {
                ui.strong("Answer match");
            }
            ui.strong("Suggested rating");
            ui.end_row();
            for row in &rows {
                review_row(run, row, (updates, scored), ui, out);
                ui.end_row();
            }
        });
}

fn review_row(
    run: &RunView,
    row: &ReviewRow,
    (updates, scored): (bool, bool),
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    ui.label(&row.title);
    ui.label(rating_text(row.previous));
    ui.label(mark_name(row.marked));
    ui.label(rating_text(row.rated));
    if scored {
        ui.label(row.answer_match.map_or_else(
            || "-".to_owned(),
            |(pct, v)| format!("{pct} % ({})", verdict_label(v)),
        ));
    }
    ui.vertical(|ui| match (row.suggestion, row.rated) {
        (Some(s), Some(rated)) => suggestion(run, row, (s, rated), updates, ui, out),
        _ => {
            ui.label("-");
        }
    });
}

fn suggestion(
    run: &RunView,
    row: &ReviewRow,
    (s, rated): (Suggestion, Rating),
    updates: bool,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    ui.label(rating_name(s.rating));
    ui.label(match s.reason {
        Reason::KnownAndFast => "This item was rather easy for you.",
        Reason::NewAndFast => "This new item went by very fast: it may be harder than it looked.",
        Reason::Slow => "This took you a long time to read: it may be harder than you rated it.",
        Reason::MarkedDifficult => {
            "You marked this item difficult, so it may be harder than you rated it."
        }
        Reason::MarkedEasy => "You marked this item easy, and it was rather easy for you.",
    });
    if let Some((words, tenths)) = row.reading {
        ui.weak(format!(
            "{words} words in {:.1} s",
            f64::from(tenths) / 10.0
        ));
    }
    if !updates {
        return;
    }
    if run.applied.contains(&row.item) {
        ui.label(format!("Changed to {}.", rating_name(s.rating)));
    } else if ui
        .button(format!(
            "Change the rating of {} to {}",
            row.title,
            rating_name(s.rating)
        ))
        .clicked()
    {
        out.push(Msg::ApplySuggestion {
            item: row.item,
            expected: rated,
            to: s.rating,
        });
    }
}
