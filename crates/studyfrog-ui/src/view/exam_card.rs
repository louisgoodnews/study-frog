//! The answer part of the exam card: one answer field and "Submit and lock answer". No show-answer, skip,
//! marks or ratings (`docs/specs/exam-mode.md`, section 7).

use eframe::egui::{self, RichText};
use studyfrog_rehearsal::case_study::CaseStudyItem;
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

/// The lines that show a case study's decision and options in an exam: the decision first, then each option,
/// then how to answer. An exam answers in writing, so the options are shown, not picked (BUG-0022).
pub fn case_lines(case: &CaseStudyItem) -> Vec<String> {
    let mut lines = vec![case.decision_prompt.trim().to_owned()];
    lines.extend(case.options.iter().map(|o| format!("- {}", o.trim())));
    lines.push("Name your choice and justify it in your written answer.".to_owned());
    lines
}

/// The decision and the options of a case study, above its answer field.
pub fn case_question(case: &CaseStudyItem, ui: &mut egui::Ui) {
    let lines = case_lines(case);
    let Some((decision, rest)) = lines.split_first() else {
        return;
    };
    ui.label(RichText::new(decision.as_str()).strong());
    if let Some((how, options)) = rest.split_last() {
        for line in options {
            ui.label(line.as_str());
        }
        ui.weak(how.as_str());
    }
    ui.add_space(6.0);
}

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
            .add_enabled_ui(ready, |ui| {
                kit::primary_button(ui, SUBMIT).on_hover_text("Submits your answer.")
            })
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn a_case_in_an_exam_shows_its_decision_and_every_option() {
        let case = CaseStudyItem {
            vignette: "A clinic needs a screen for anxiety in its intake.".to_owned(),
            decision_prompt: "Which instrument should the clinic use?".to_owned(),
            options: vec![
                "Short screen".to_owned(),
                "Full interview".to_owned(),
                "Self-report scale".to_owned(),
            ],
            expected_choices: vec!["Short screen".to_owned()],
            rubric_criteria: vec!["addresses the time available".to_owned()],
        };
        let lines = case_lines(&case);
        assert_eq!(lines[0], "Which instrument should the clinic use?");
        assert_eq!(lines[1], "- Short screen");
        assert_eq!(lines[3], "- Self-report scale");
        assert!(lines.last().is_some_and(|l| l.contains("written answer")));
    }
}
