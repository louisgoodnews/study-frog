//! The mock exam before it starts: the summary, the questions by section, and the reference answers when shown
//! (STORY-0186, STORY-0241). The same text and the same buttons as before, on redesign cards.

use eframe::egui::{self, RichText};
use studyfrog_rehearsal::exam_answer::{choice_tag, format_expected_text};
use studyfrog_rehearsal::exam_format::{ExamItemFormat, format_label, resolved_mix_total};
use studyfrog_rehearsal::mock_exam::{ExamQuestion, all_questions};
use studyfrog_service::exam_gen::Generated;

use crate::msg::Msg;
use crate::setup::mock_exam::{MockExamMsg, MockExamUi};
use crate::theme::palette_of;
use crate::view::{home_hero, kit};

/// Draw the exam before it starts: the summary, the questions and the buttons that start, save or show the answers.
pub fn draw(ui: &mut egui::Ui, g: &Generated, m: &MockExamUi, out: &mut Vec<Msg>) {
    summary(ui, g, m);
    let mut n = 0;
    for section in &g.exam.sections {
        ui.add_space(6.0);
        home_hero::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(&section.name).strong());
            for q in &section.questions {
                n += 1;
                question(ui, n, q);
            }
        });
    }
    ui.add_space(8.0);
    if m.show_answers {
        reference_answers(ui, g);
    }
    buttons(ui, m, out);
    if let Some(path) = &m.saved {
        ui.weak(format!("Saved as {path}"));
    }
}

/// The counts and the warnings above the questions.
fn summary(ui: &mut egui::Ui, g: &Generated, m: &MockExamUi) {
    let wanted_total = resolved_mix_total(&m.format_mix);
    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(format!(
            "{} questions, {} minutes, {} points: {} from the model, {} from the notes' templates.",
            g.keys.len(),
            g.exam.time_limit_min,
            studyfrog_rehearsal::mock_exam::total_points(&g.exam),
            g.from_model,
            g.from_templates
        ));
        if g.keys.len() < wanted_total {
            ui.weak(format!(
                "The notes supplied {} of the {} questions asked for. Add more definitions, formulas or cases to the notes to fill it.",
                g.keys.len(),
                wanted_total
            ));
        }
        for s in &g.short {
            ui.label(studyfrog_rehearsal::curriculum_build::short_note(s));
        }
        if !g.rejected.is_empty() {
            ui.weak(format!(
                "{} questions were left out because the notes do not support them.",
                g.rejected.len()
            ));
        }
    });
}

/// One question: its number, points, topic and prompt, with its options when it has them.
fn question(ui: &mut egui::Ui, n: usize, q: &ExamQuestion) {
    let pal = palette_of(ui);
    ui.horizontal_wrapped(|ui| {
        kit::pill(ui, format_label(q.format), pal.accent);
        ui.label(format!(
            "{n}. ({} points, {}) {}",
            q.points.0, q.topic, q.prompt
        ));
    });
    options(ui, q);
}

fn options(ui: &mut egui::Ui, q: &ExamQuestion) {
    let mark = match q.format {
        ExamItemFormat::SingleSelect => "( )",
        ExamItemFormat::MultipleSelect => "[ ]",
        _ => return,
    };
    for (idx, opt) in q.options.iter().enumerate() {
        ui.weak(format!("   {mark} {}. {opt}", choice_tag(idx)));
    }
}

fn reference_answers(ui: &mut egui::Ui, g: &Generated) {
    let questions = all_questions(&g.exam);
    ui.label(RichText::new("Reference answers").strong());
    for (i, k) in g.keys.iter().enumerate() {
        let opts = questions.get(i).map_or(&[][..], |q| q.options.as_slice());
        ui.label(format!("{}. {}", i + 1, format_expected_text(k, opts)));
        ui.weak(format!("Points to make: {}", k.rubric.join("; ")));
    }
}

fn buttons(ui: &mut egui::Ui, m: &MockExamUi, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        if kit::primary_button(ui, "Start the exam")
            .on_hover_text(
                "Starts writing the answers. Each answer is graded against its question's points.",
            )
            .clicked()
        {
            out.push(Msg::MockExam(MockExamMsg::Start));
        }
        if kit::quiet_button(ui, "Save as note")
            .on_hover_text("Saves the exam into the stack as a note, with its reference answers.")
            .clicked()
        {
            out.push(Msg::MockExam(MockExamMsg::Save));
        }
        let toggle = if m.show_answers {
            "Hide reference answers"
        } else {
            "Show reference answers"
        };
        if kit::quiet_button(ui, toggle)
            .on_hover_text("Shows or hides the reference answers and their points.")
            .clicked()
        {
            out.push(Msg::MockExam(MockExamMsg::ToggleAnswers));
        }
    });
}
