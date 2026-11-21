//! The end of a memory game: the victory summary with the personal best, or the timeout line, and
//! the button to play again.

use eframe::egui;
use studyfrog_rehearsal::memory::{MemoryOutcome, MemorySession};
use studyfrog_rehearsal::memory_summary::{defeat_message, victory_summary};

use crate::msg::Msg;
use crate::setup::memory::{MemoryMsg, MemoryUi};
use crate::view::kit;
use crate::view::memory_board::clock_text;

/// How the game ended: the victory summary with the personal best, or the encouraging timeout line,
/// each with a button to play again.
pub(super) fn end_line(
    ui: &mut egui::Ui,
    s: &MemorySession,
    memory: &MemoryUi,
    out: &mut Vec<Msg>,
) {
    let Some(outcome) = s.outcome else {
        return;
    };
    match outcome {
        MemoryOutcome::Completed { .. } => {
            if let Some(v) = victory_summary(s, memory.timer, memory.prior_best_ms) {
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
                lines.push(match (v.personal_best, memory.prior_best_ms) {
                    (true, Some(_)) => "A new personal best!".to_owned(),
                    (true, None) => "Your first time on record.".to_owned(),
                    (false, Some(best)) => format!("Your best is {}.", clock_text(best)),
                    (false, None) => String::new(),
                });
                for line in lines.iter().filter(|l| !l.is_empty()) {
                    ui.label(crate::typo::text(ui, line.clone(), crate::typo::Tier::Body));
                }
            }
        }
        MemoryOutcome::TimedOut => {
            ui.label(crate::typo::text(
                ui,
                defeat_message(),
                crate::typo::Tier::Body,
            ));
        }
    }
    if kit::primary_button(ui, "Play again")
        .on_hover_text("Shuffles the same cards into a new game.")
        .clicked()
    {
        out.push(Msg::Memory(MemoryMsg::Start));
    }
}
