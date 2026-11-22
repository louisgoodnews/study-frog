//! The mock exam view (STORY-0186, STORY-0187): the exam made from the notes, the answers written, the
//! grading, and saving the exam as a note.

mod result;

use eframe::egui::{self, RichText, TextEdit};

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
    if let Some(graded) = &m.graded {
        result::draw(ui, graded);
    } else if let Some(g) = &m.result {
        if m.started {
            answering(ui, g, m, out);
        } else {
            preview(ui, g, m, out);
        }
    }
    if kit::quiet_button(ui, "Close exam")
        .on_hover_text("Closes the exam and returns to the setup.")
        .clicked()
    {
        out.push(Msg::MockExam(MockExamMsg::Close));
    }
}

fn preview(
    ui: &mut egui::Ui,
    g: &studyfrog_service::exam_gen::Generated,
    m: &MockExamUi,
    out: &mut Vec<Msg>,
) {
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
    if let Some(path) = &m.saved {
        ui.weak(format!("Saved as {path}"));
    }
}

fn answering(
    ui: &mut egui::Ui,
    g: &studyfrog_service::exam_gen::Generated,
    m: &MockExamUi,
    out: &mut Vec<Msg>,
) {
    for (i, q) in g
        .exam
        .sections
        .iter()
        .flat_map(|s| s.questions.iter())
        .enumerate()
    {
        ui.add_space(6.0);
        ui.label(format!(
            "{}. ({} points, {}) {}",
            i + 1,
            q.points.0,
            q.topic,
            q.prompt
        ));
        let mut text = m.answers.get(i).cloned().unwrap_or_default();
        let field = ui.add(
            TextEdit::multiline(&mut text)
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        );
        field.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::TextInput, true, format!("Answer {}", i + 1))
        });
        if field.changed() {
            out.push(Msg::MockExam(MockExamMsg::Answer(i, text)));
        }
    }
    ui.add_space(8.0);
    if kit::primary_button(ui, "Finish and grade")
        .on_hover_text("Grades every answer and shows the grade with the review.")
        .clicked()
    {
        out.push(Msg::MockExam(MockExamMsg::Finish));
    }
}
