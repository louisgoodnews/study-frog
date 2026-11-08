//! The answer part of the exam card: one answer field and "Submit and lock answer". No show-answer, skip,
//! marks or ratings (`docs/specs/exam-mode.md`, section 7).

use eframe::egui;
use studyfrog_rehearsal::input::Action;

use crate::exam_ui::answers;
use crate::msg::Msg;
use crate::run_state::RunView;
use crate::run_update::awaiting_reveal;
use crate::view::kit;
use crate::view::widgets::text_field;

/// The label of the submit button.
pub const SUBMIT: &str = "Submit and lock answer";

/// The rules, as the exam states them under the card.
pub const EXAM_HINTS: &str = "Exam: write an answer to go on  ·  a submitted answer is locked  ·  no skipping, \
     no going back, no answers shown until the end  ·  P pause";

/// The answer field and the submit button.
pub fn controls(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if !awaiting_reveal(run) {
        return;
    }
    if let Some(text) = text_field(ui, "Your answer", &run.answer, "write your answer", 4) {
        out.push(Msg::AnswerEdited(text));
    }
    let ready = !run.answer.trim().is_empty();
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let submit = ui
            .add_enabled_ui(ready, |ui| kit::primary_button(ui, SUBMIT))
            .inner;
        if submit.clicked() {
            out.push(Msg::RunAction(Action::SubmitAnswer(run.answer.clone())));
        }
        let done = answers(run).len();
        ui.weak(format!("{done} of {} answered", run.session.queue.len()));
    });
    if !ready {
        ui.weak("Write an answer to submit it; you cannot skip an item.");
    }
}
