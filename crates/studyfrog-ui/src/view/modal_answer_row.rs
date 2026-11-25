//! The answer type of a new question in the Create dialog (STORY-0223): open text, single choice or multiple select.

use eframe::egui;

use crate::modal::{CreateForm, ModalMsg};
use crate::msg::Msg;
use crate::view::choice_body::{AnswerType, answer_type_label};

const CHOICES: [(AnswerType, &str); 3] = [
    (AnswerType::Open, "The answer is typed in the reference answer."),
    (
        AnswerType::Single,
        "One option is correct. Each option is a line `- [ ] text` or `- [x] text`.",
    ),
    (
        AnswerType::Multiple,
        "Two or more options are correct. The options are written like single choice.",
    ),
];

/// The row of answer types under the question's prompt.
pub fn row(f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        ui.weak("Answer");
        for (kind, hover) in CHOICES {
            let selected = f.answer_type == kind;
            if ui
                .selectable_label(selected, answer_type_label(kind))
                .on_hover_text(hover)
                .clicked()
            {
                out.push(Msg::Modal(ModalMsg::SetAnswerType(kind)));
            }
        }
    });
}
