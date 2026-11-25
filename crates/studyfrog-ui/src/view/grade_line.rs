//! The grade line of a choice answer: the credit it earned, the rule that gave it and the suggested
//! rating (STORY-0231).

use eframe::egui::Ui;
use studyfrog_service::choice::choice_rule_text;

use crate::run_state::Scored;

/// Show the grade line of a choice answer and return true; other answers leave `ui` untouched and
/// return false.
pub fn choice_line(scored: &Scored, ui: &mut Ui) -> bool {
    let Some(rule) = choice_rule_text(&scored.engine) else {
        return false;
    };
    ui.label(format!(
        "Your answer earned {:.0} % ({rule}). Suggested rating: {:?}.",
        scored.score.value * 100.0,
        scored.rating
    ));
    true
}
