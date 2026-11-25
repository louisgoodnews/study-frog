//! The grading-scheme selector for choice questions on the settings screen (STORY-0231).

use eframe::egui::Ui;
use studyfrog_core::choice_grading::ChoiceGrading;

use crate::msg::{Msg, MsgVault};
use crate::state::State;

/// The three schemes, each with its label and the tooltip that explains it.
const SCHEMES: [(ChoiceGrading, &str, &str); 3] = [
    (
        ChoiceGrading::AllOrNothing,
        "All or nothing",
        "Full credit only when the chosen options are exactly the correct ones. Anything else earns nothing.",
    ),
    (
        ChoiceGrading::Zapechelnyuk,
        "Zapechelnyuk",
        "Zapechelnyuk (2015): a choice earns 1/k of the credit when at least one chosen option is correct, k being the number of options chosen. Choosing more options dilutes the credit.",
    ),
    (
        ChoiceGrading::CountPartial,
        "Count-based partial",
        "Correct options chosen minus wrong options chosen, divided by the number of correct options. Never below zero.",
    ),
];

/// The row that chooses how choice questions earn credit.
pub fn grading_row(state: &State, ui: &mut Ui, out: &mut Vec<Msg>) {
    ui.label("Choice questions earn credit by:");
    ui.horizontal(|ui| {
        for (scheme, label, tip) in SCHEMES {
            let selected = state.app.choice_grading == scheme;
            if ui
                .selectable_label(selected, label)
                .on_hover_text(tip)
                .clicked()
            {
                out.push(Msg::Vault(MsgVault::SetChoiceGrading(scheme)));
            }
        }
    });
}
