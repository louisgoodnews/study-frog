//! The end of an exam: the percentage, the grade badge, passed or not, and a review of every item with the
//! written answer, the reference and the score (`docs/specs/exam-mode.md`, section 7).

use eframe::egui::{self, RichText};
use studyfrog_core::exam::{ExamResult, grade_badge, grade_words};
use studyfrog_core::run::RunState;
use studyfrog_core::uid::Uid;
use studyfrog_service::exam::exam_items;

use crate::exam_ui::{answer_of, answers};
use crate::msg::Msg;
use crate::run_state::RunView;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::summary::verdict_label;

/// Heading, the result banner, the review and the diary prompt.
pub fn draw(state: &State, run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    crate::view::summary::shortcuts(run, ui, out);
    egui::ScrollArea::vertical()
        .id_salt("exam-result-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let stopped = run.session.run.state == RunState::Aborted;
            ui.horizontal(|ui| {
                let title = if stopped {
                    "Exam stopped"
                } else {
                    "Exam finished"
                };
                ui.label(RichText::new(title).size(26.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if kit::primary_button(ui, "Back to setup").clicked() {
                        out.push(Msg::LeaveRun);
                    }
                });
            });
            banner(run, stopped, ui);
            ui.weak("An exam never changes your review schedule.");
            ui.add_space(4.0);
            kit::section(ui, "Review");
            for item in exam_items(&run.session.queue) {
                item_card(run, item, ui);
            }
            ui.add_space(4.0);
            crate::view::run_diary_prompt::draw(state, run, ui, out);
        });
}

fn banner(run: &RunView, stopped: bool, ui: &mut egui::Ui) {
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        if stopped {
            ui.label("You stopped the exam, so it is not graded. Your answers are kept in the run's history.");
        } else if let Some(r) = &run.exam.result {
            result_banner(r, ui);
        } else if let Some(e) = &run.exam.error {
            ui.label(format!("The exam could not be graded: {e}"));
        } else {
            let given = answers(run).len();
            let scored = run.scores.len() + run.exam.unscored.len();
            ui.label(RichText::new("Grading…").size(20.0).strong());
            ui.weak(format!("{scored} of {given} answers compared"));
        }
    });
}

fn result_banner(r: &ExamResult, ui: &mut egui::Ui) {
    let p = palette_of(ui);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{:.1} %", r.percent))
                .size(40.0)
                .strong(),
        );
        ui.add_space(16.0);
        ui.label(
            RichText::new(grade_badge(r.grade))
                .size(28.0)
                .strong()
                .color(p.accent),
        );
        ui.add_space(16.0);
        let (text, color) = if r.passed {
            ("Passed", p.success)
        } else {
            ("Not passed", p.danger)
        };
        kit::pill(ui, text, color);
    });
    ui.label(grade_words(r.grade));
    ui.weak(format!(
        "{} of {} items answered  ·  60 % needed to pass  ·  compared by {}",
        r.answered,
        r.items,
        short_engine(&r.engine)
    ));
}

/// The runtime's name without the reason a fallback was taken (that reason is in the log and the notice).
fn short_engine(engine: &str) -> &str {
    let second = engine.match_indices(" (").nth(1).map(|(i, _)| i);
    match (engine.is_empty(), second) {
        (true, _) => "-",
        (false, Some(i)) => &engine[..i],
        (false, None) => engine,
    }
}

fn item_card(run: &RunView, item: Uid, ui: &mut egui::Ui) {
    let p = palette_of(ui);
    let content = run.content.get(&item);
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        let title = content.map_or("(not loaded)", |c| c.title.as_str());
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(title).strong());
            score_pill(run, item, ui);
        });
        match answer_of(run, item) {
            Some(answer) => ui.label(format!("Your answer: {answer}")),
            None => ui.colored_label(p.danger, "Not answered (counts 0)"),
        };
        if let Some(c) = content {
            ui.label(RichText::new(format!("Reference: {}", c.answer)).color(p.accent));
        }
    });
}

fn score_pill(run: &RunView, item: Uid, ui: &mut egui::Ui) {
    let p = palette_of(ui);
    if let Some(s) = run.scores.get(&item) {
        let pct = (s.score.value * 100.0).round();
        kit::pill(
            ui,
            &format!("{pct:.0} % ({})", verdict_label(s.score.verdict)),
            p.accent,
        );
    } else if run.exam.unscored.contains(&item) {
        kit::pill(ui, "could not be compared (0 %)", p.danger);
    } else if answer_of(run, item).is_some() {
        kit::pill(ui, "comparing…", p.text_weak);
    }
}
