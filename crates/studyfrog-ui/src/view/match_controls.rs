//! Scope picker, filter selector, pair-count controls, HUD, and honest empty state for Match mode (EPIC-075).
//!
//! Complies with Invariant P1: zero inherent impl blocks; all logic in free functions.

use eframe::egui::{self, ComboBox, RichText, Role, WidgetInfo};
use studyfrog_rehearsal::match_mode::format_match_time;
use studyfrog_service::match_pairs::MatchFilter;

use crate::match_ui::{MatchMsg, MatchUiState};
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::Palette;
use crate::view::{home_hero, kit};

const FILTERS: [(MatchFilter, &str); 3] = [
    (MatchFilter::All, "All"),
    (MatchFilter::DueFirst, "Due first"),
    (MatchFilter::LeastFamiliar, "Least familiar"),
];

const PAIR_COUNTS: [usize; 3] = [4, 6, 8];

/// Scope, filter, and pair-count controls bar.
pub(super) fn draw_controls(
    state: &State,
    ms: &MatchUiState,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    p: &Palette,
) {
    let stacks: Vec<_> = state
        .vault
        .iter()
        .flat_map(|v| v.overview.stacks.iter())
        .collect();
    let scope_label = ms
        .stack_path
        .as_deref()
        .and_then(|folder| stacks.iter().find(|s| s.folder == folder))
        .map_or_else(|| "Entire vault".to_owned(), |s| s.title.clone());

    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("Scope")
                    .size(crate::typo::size(ui, crate::typo::Tier::Label))
                    .color(p.text_weak),
            );
            let combo = ComboBox::from_id_salt("match-scope")
                .selected_text(scope_label)
                .width(180.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(ms.stack_path.is_none(), "Entire vault")
                        .clicked()
                    {
                        out.push(Msg::Match(MatchMsg::SetScope(None)));
                    }
                    for s in &stacks {
                        let count = s.cards + s.questions;
                        let text = format!("{} ({count})", s.title);
                        let active = ms.stack_path.as_deref() == Some(s.folder.as_str());
                        if ui.selectable_label(active, text).clicked() {
                            out.push(Msg::Match(MatchMsg::SetScope(Some(s.folder.clone()))));
                        }
                    }
                });
            combo
                .response
                .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, "Match scope"));

            ui.separator();
            for (filter, label) in FILTERS {
                if ui
                    .selectable_label(ms.filter == filter, label)
                    .on_hover_text("Orders which vault pairs are prioritized for Match rounds.")
                    .clicked()
                {
                    out.push(Msg::Match(MatchMsg::SetFilter(filter)));
                }
            }

            ui.separator();
            for count in PAIR_COUNTS {
                let label = format!("{count} pairs");
                if ui
                    .selectable_label(ms.pair_count == count, label)
                    .on_hover_text("Sets how many pairs are dealt into each round.")
                    .clicked()
                {
                    out.push(Msg::Match(MatchMsg::SetPairCount(count)));
                }
            }

            ui.separator();
            let avail = format!("{} available", ms.pool.len());
            ui.label(
                RichText::new(avail)
                    .size(crate::typo::size(ui, crate::typo::Tier::Label))
                    .color(p.text_weak),
            );
        });
    });
}

/// Top HUD row: matched count, monotonic timer, streak, Restart, New round, and Exit.
pub(super) fn draw_hud(ms: &MatchUiState, ui: &mut egui::Ui, out: &mut Vec<Msg>, p: &Palette) {
    let session = &ms.session;
    ui.horizontal(|ui| {
        let total = session.tiles.len() / 2;
        let matched = total.saturating_sub(session.pairs_remaining);
        ui.label(
            RichText::new(format!("Pairs: {matched}/{total}"))
                .size(crate::typo::size(ui, crate::typo::Tier::Label))
                .strong(),
        );

        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.label(
                    RichText::new(format_match_time(session.elapsed_ms))
                        .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                        .strong(),
                );
            },
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button("Exit")
                .on_hover_text("Leaves Match mode.")
                .clicked()
            {
                out.push(Msg::Match(MatchMsg::Leave));
            }
            if ui
                .button("New round")
                .on_hover_text("Deals a fresh set of pairs from the selected scope.")
                .clicked()
            {
                out.push(Msg::Match(MatchMsg::NewRound));
            }
            if ui
                .button("Restart")
                .on_hover_text("Reshuffles the current round.")
                .clicked()
            {
                out.push(Msg::Match(MatchMsg::Restart));
            }
            let streak_color = if session.streak > 1 {
                p.accent
            } else {
                p.text_weak
            };
            ui.label(
                RichText::new(format!("Streak: {}x", session.streak))
                    .size(crate::typo::size(ui, crate::typo::Tier::Label))
                    .color(streak_color)
                    .strong(),
            );
        });
    });
}

/// Honest loading, error, or empty state when fewer than 2 valid vault pairs exist.
pub(super) fn draw_empty_state(
    ms: &MatchUiState,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    p: &Palette,
) {
    ui.add_space(16.0);
    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        if ms.loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading flashcards and questions from your vault...");
            });
            return;
        }
        if let Some(err) = &ms.error {
            ui.label(
                RichText::new("Could not load Match pairs")
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                    .color(p.danger)
                    .strong(),
            );
            ui.add_space(4.0);
            ui.label(err);
            return;
        }
        ui.label(
            RichText::new("Not enough cards or questions to match")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong(),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "Match mode needs at least 2 flashcards or questions with both a prompt and an answer in the selected scope.",
            )
            .color(p.text_weak),
        );
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            if ms.stack_path.is_some()
                && ui
                    .button("Use entire vault")
                    .on_hover_text("Switches Match scope to include every stack in this vault.")
                    .clicked()
            {
                out.push(Msg::Match(MatchMsg::SetScope(None)));
            }
            if kit::primary_button(ui, "Create an item")
                .on_hover_text("Opens the dialog to add a new flashcard or question.")
                .clicked()
            {
                out.push(Msg::Modal(ModalMsg::OpenCreate));
            }
        });
    });
}
