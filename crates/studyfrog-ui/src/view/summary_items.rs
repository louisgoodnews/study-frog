//! One card of the run review: how the item went and what the program suggests.

use eframe::egui::{self, RichText};
use studyfrog_core::review::Rating;
use studyfrog_rehearsal::reading::{Reason, Suggestion};

use crate::msg::Msg;
use crate::review::ReviewRow;
use crate::run_state::RunView;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::summary::{mark_name, rating_name, rating_text, verdict_label};

/// The title, the previous rating, your mark, your rating, the answer match and the suggestion.
pub fn card(run: &RunView, row: &ReviewRow, updates: bool, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        ui.label(RichText::new(&row.title).strong());
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Previous rating: {}", rating_text(row.previous)));
            ui.label(format!("Your mark: {}", mark_name(row.marked)));
            ui.label(format!("Your rating: {}", rating_text(row.rated)));
            if row.suggestion.is_none() {
                ui.label(format!("Suggested rating: {}", rating_text(row.rated)));
                if let Some((words, tenths)) = row.reading {
                    ui.weak(format!(
                        "{words} words in {:.1} s",
                        f64::from(tenths) / 10.0
                    ));
                }
            }
            if let Some((pct, v)) = row.answer_match {
                kit::pill(ui, &format!("{pct} % ({})", verdict_label(v)), p.accent);
            }
        });
        if let (Some(s), Some(rated)) = (row.suggestion, row.rated) {
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!("Suggested rating: {}", rating_name(s.rating))).strong(),
            );
            suggestion(run, row, (s, rated), updates, ui, out);
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
