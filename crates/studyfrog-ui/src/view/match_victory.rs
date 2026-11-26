//! The victory summary of Match Mode in the redesign style (EPIC-073, STORY-0237): the result on a redesign card,
//! with the same figures, the same messages and the same two actions as before.

use eframe::egui::{self, RichText};
use studyfrog_rehearsal::match_mode::{MatchModeSession, compute_match_score, format_match_time};

use crate::match_ui::MatchMsg;
use crate::msg::Msg;
use crate::theme::Palette;
use crate::view::home_hero;

/// The completed match: the final time and score, the run's figures and the Play Again and Done actions.
pub(super) fn draw(
    session: &MatchModeSession,
    high_score: u32,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    p: &Palette,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.label(
            RichText::new("Match Complete!")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .color(p.success)
                .strong(),
        );
        ui.add_space(8.0);
        result_card(session, high_score, p, ui);
        ui.add_space(20.0);
        actions(ui, out);
    });
}

/// The final time, the score and the run's figures on one card.
fn result_card(session: &MatchModeSession, high_score: u32, p: &Palette, ui: &mut egui::Ui) {
    let score = compute_match_score(session);
    home_hero::card(ui, |ui| {
        ui.label(
            RichText::new(format!(
                "Final Time: {}",
                format_match_time(session.elapsed_ms)
            ))
            .size(crate::typo::size(ui, crate::typo::Tier::Heading))
            .strong(),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("Score: {score} pts"))
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .color(p.accent)
                .strong(),
        );
        ui.add_space(12.0);
        ui.label(format!("Best Streak: {}x", session.best_streak));
        ui.label(format!("Mismatches: {}", session.mismatches));
        if high_score > 0 {
            ui.label(format!("Session High Score: {high_score} pts"));
        }
    });
}

/// The New round, Play Again, and Done actions.
fn actions(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.add_space(ui.available_width() * 0.25);
        if crate::view::kit::primary_button(ui, "New round")
            .on_hover_text("Deals a fresh set of pairs from the selected scope.")
            .clicked()
        {
            out.push(Msg::Match(MatchMsg::NewRound));
        }
        if ui
            .button("Play Again")
            .on_hover_text("Reshuffles and replays the same round.")
            .clicked()
        {
            out.push(Msg::Match(MatchMsg::Restart));
        }
        if ui
            .button("Done")
            .on_hover_text("Leaves Match mode.")
            .clicked()
        {
            out.push(Msg::Match(MatchMsg::Leave));
        }
    });
}
