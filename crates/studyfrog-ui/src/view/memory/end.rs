//! The end of a memory game: the victory summary (in a dialog, and as a short line under the HUD)
//! with the personal best, and the timeout line. Each end offers a button to play again.

use eframe::egui;
use studyfrog_rehearsal::memory::{MemoryOutcome, MemorySession};
use studyfrog_rehearsal::memory_summary::{VictorySummary, defeat_message};

use crate::msg::Msg;
use crate::setup::memory::MemoryMsg;
use crate::view::kit;
use crate::view::memory::board::clock_text;

/// The lines of a victory summary; the personal-best line depends on the best before this game.
pub fn summary_lines(v: &VictorySummary, prior_best_ms: Option<u64>) -> Vec<String> {
    let mut lines = vec![format!(
        "All pairs found in {}. Moves {}, accuracy {:.0}%, speed {:.1} pairs a minute, best streak {}.",
        clock_text(v.elapsed_ms),
        v.moves,
        v.accuracy_pct,
        v.speed_score,
        v.best_streak
    )];
    if v.bonus_points > 0 {
        lines.push(format!("Time bonus: {} points.", v.bonus_points));
    }
    lines.push(match (v.personal_best, prior_best_ms) {
        (true, Some(_)) => "A new personal best!".to_owned(),
        (true, None) => "Your first time on record.".to_owned(),
        (false, Some(best)) => format!("Your best is {}.", clock_text(best)),
        (false, None) => String::new(),
    });
    lines.retain(|l| !l.is_empty());
    lines
}

/// Under the HUD: the short result of a finished game, or the timeout line, each with a play-again
/// button. The full summary is in the dialog that opens when a game is completed.
pub(super) fn end_line(ui: &mut egui::Ui, s: &MemorySession, out: &mut Vec<Msg>) {
    let Some(outcome) = s.outcome else {
        return;
    };
    match outcome {
        MemoryOutcome::Completed { elapsed_ms } => {
            ui.label(crate::typo::text(
                ui,
                format!("All pairs found in {}.", clock_text(elapsed_ms)),
                crate::typo::Tier::Body,
            ));
        }
        MemoryOutcome::TimedOut => {
            ui.label(crate::typo::text(
                ui,
                defeat_message(),
                crate::typo::Tier::Body,
            ));
        }
    }
    play_again(ui, out);
}

/// The victory summary in a dialog: the lines, then play again or close.
pub fn summary_dialog(
    v: &VictorySummary,
    prior_best_ms: Option<u64>,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    ui.label(crate::typo::text(ui, "Memory game complete", crate::typo::Tier::Heading).strong());
    ui.add_space(8.0);
    for line in summary_lines(v, prior_best_ms) {
        ui.label(crate::typo::text(ui, line, crate::typo::Tier::Body));
    }
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        if kit::primary_button(ui, "Play again")
            .on_hover_text("Shuffles the same cards into a new game.")
            .clicked()
        {
            out.push(Msg::Modal(crate::modal_types::ModalMsg::Close));
            out.push(Msg::Memory(MemoryMsg::Start));
        }
        if kit::quiet_button(ui, "Close")
            .on_hover_text("Closes the summary and keeps the finished board.")
            .clicked()
        {
            out.push(Msg::Modal(crate::modal_types::ModalMsg::Close));
        }
    });
}

fn play_again(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if kit::primary_button(ui, "Play again")
        .on_hover_text("Shuffles the same cards into a new game.")
        .clicked()
    {
        out.push(Msg::Memory(MemoryMsg::Start));
    }
}
