//! Settings: whether familiarity improves the predictions of recall on the user's own history. Advice only.

use eframe::egui;
use studyfrog_report::replay::{MIN_REVIEWS, ReplayReport, Verdict};

use crate::state::State;

fn verdict_text(r: &ReplayReport) -> String {
    let fitted = format!(
        "strength {}, half-stability {} days",
        r.used.strength, r.used.s_half
    );
    match r.verdict {
        Verdict::TooFewReviews => format!(
            "Too few reviews for a verdict yet ({} of {MIN_REVIEWS} counted). Familiarity is on with its default values.",
            r.scored
        ),
        Verdict::FactorHelps => format!(
            "On your own history, familiarity improves the predictions of recall. Best values found: {fitted}."
        ),
        Verdict::PlainBetter => format!(
            "On your own history, plain FSRS predicts your recall better. You can switch familiarity off with familiarity_strength = 0 in studyfrog.toml, or use the best values found: {fitted}. Nothing is changed for you."
        ),
        Verdict::Similar => {
            "On your own history, familiarity makes no clear difference to the predictions."
                .to_owned()
        }
    }
}

/// The section.
pub fn draw(state: &State, ui: &mut egui::Ui) {
    crate::view::kit::section(ui, "Does familiarity help?");
    let Some(b) = &state.reports else {
        ui.weak("Checking your review history…");
        return;
    };
    let r = &b.replay;
    ui.label(verdict_text(r));
    if r.scored == 0 {
        return;
    }
    ui.weak(format!(
        "Log loss on the tested part (lower is better): plain {:.3}, with familiarity {:.3}.",
        r.loss.0, r.loss.1
    ));
    let more = r.workload_percent >= 0.0;
    ui.weak(format!(
        "With these values you would review about {:.0} % {} than without familiarity.",
        r.workload_percent.abs(),
        if more { "more" } else { "less" }
    ));
}
