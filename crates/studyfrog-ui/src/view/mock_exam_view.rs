//! The mock exam view (STORY-0186): the exam that was made from the notes, its reference answers on
//! request, and saving it as a note.

use eframe::egui::{self, RichText};

use crate::msg::Msg;
use crate::setup::mock_exam::{MockExamMsg, MockExamUi};
use crate::theme::palette_of;
use crate::view::kit;

/// Draw the exam view.
pub fn draw(ui: &mut egui::Ui, m: &MockExamUi, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    ui.label(RichText::new(&m.title).strong());
    if m.loading {
        ui.label("Writing the exam from your notes. This can take a while with a local model.");
    }
    if let Some(error) = &m.error {
        ui.colored_label(pal.danger, error);
    }
    if let Some(g) = &m.result {
        ui.label(format!(
            "{} questions, {} minutes, {} points: {} from the model, {} from the notes' templates.",
            g.keys.len(),
            g.exam.time_limit_min,
            studyfrog_rehearsal::mock_exam::total_points(&g.exam),
            g.from_model,
            g.from_templates
        ));
        if g.keys.len() < crate::setup::mock_exam::EXAM_QUESTIONS {
            ui.weak(format!(
                "The notes supplied {} of the {} questions asked for. Add more definitions, formulas or cases to the notes to fill it.",
                g.keys.len(),
                crate::setup::mock_exam::EXAM_QUESTIONS
            ));
        }
        if !g.rejected.is_empty() {
            ui.weak(format!(
                "{} questions were left out because the notes do not support them.",
                g.rejected.len()
            ));
        }
        let mut n = 0;
        for section in &g.exam.sections {
            ui.add_space(6.0);
            ui.label(RichText::new(&section.name).strong());
            for q in &section.questions {
                n += 1;
                ui.label(format!(
                    "{n}. ({} points, {}) {}",
                    q.points.0, q.topic, q.prompt
                ));
            }
        }
        ui.add_space(8.0);
        if m.show_answers {
            ui.label(RichText::new("Reference answers").strong());
            for (i, k) in g.keys.iter().enumerate() {
                ui.label(format!("{}. {}", i + 1, k.sample_answer));
                ui.weak(format!("Points to make: {}", k.rubric.join("; ")));
            }
        }
        ui.horizontal(|ui| {
            if kit::primary_button(ui, "Save as note")
                .on_hover_text(
                    "Saves the exam into the stack as a note, with its reference answers.",
                )
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
        if let Some(path) = &m.saved {
            ui.weak(format!("Saved as {path}"));
        }
    }
    if kit::quiet_button(ui, "Close exam")
        .on_hover_text("Closes the exam and returns to the setup.")
        .clicked()
    {
        out.push(Msg::MockExam(MockExamMsg::Close));
    }
}
