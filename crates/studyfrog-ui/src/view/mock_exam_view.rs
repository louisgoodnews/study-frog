//! The mock exam view (STORY-0186, STORY-0187, STORY-0207): slide picker, format-aware preview,
//! answering, graded review, and saving as a note.

mod answering;
mod result;

use eframe::egui::{self, RichText};
use studyfrog_rehearsal::exam_answer::{choice_tag, format_expected_text};
use studyfrog_rehearsal::exam_format::{ExamItemFormat, format_label, resolved_mix_total};
use studyfrog_rehearsal::mock_exam::all_questions;

use crate::msg::Msg;
use crate::setup::mock_exam::{MockExamMsg, MockExamUi, SlideChoice};
use crate::theme::palette_of;
use crate::view::kit;

/// Draw the exam view.
pub fn draw(ui: &mut egui::Ui, m: &MockExamUi, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    ui.label(RichText::new(&m.title).strong());
    if m.loading {
        ui.label("Writing the exam. This can take a while with a local model.");
    }
    if m.judging {
        ui.label("Grading your answers with the local model. This can take a while.");
    }
    if let Some(note) = &m.note {
        ui.weak(note);
    }
    if let Some(error) = &m.error {
        ui.colored_label(pal.danger, error);
    }
    if let Some(graded) = &m.graded {
        result::draw(
            ui,
            graded,
            crate::setup::mock_exam::grading::pass_mark_of(m),
            out,
        );
    } else if let Some(g) = &m.result {
        if m.started {
            answering::draw(ui, g, m, out);
        } else {
            preview(ui, g, m, out);
        }
    } else if let Some(slides) = &m.slides {
        choose_slides(ui, slides, out);
    }
    if kit::quiet_button(ui, "Close exam")
        .on_hover_text("Closes the exam and returns to the setup.")
        .clicked()
    {
        out.push(Msg::MockExam(MockExamMsg::Close));
    }
}

fn choose_slides(ui: &mut egui::Ui, slides: &[SlideChoice], out: &mut Vec<Msg>) {
    if slides.is_empty() {
        ui.weak(
            "No readable PDF is in the vault. Add the lecture slides as PDF files, then try again.",
        );
        return;
    }
    ui.label("Choose the lecture PDF to make the exam from.");
    for slide in slides {
        if kit::quiet_button(ui, &slide.title)
            .on_hover_text(format!(
                "Configure formats and make an exam from {}.",
                slide.path
            ))
            .clicked()
        {
            out.push(Msg::MockExam(MockExamMsg::FromSlide(slide.path.clone())));
        }
    }
}

fn preview(
    ui: &mut egui::Ui,
    g: &studyfrog_service::exam_gen::Generated,
    m: &MockExamUi,
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    let wanted_total = resolved_mix_total(&m.format_mix);
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
    let mut n = 0;
    for section in &g.exam.sections {
        ui.add_space(6.0);
        ui.label(RichText::new(&section.name).strong());
        for q in &section.questions {
            n += 1;
            ui.horizontal_wrapped(|ui| {
                kit::pill(ui, format_label(q.format), pal.accent);
                ui.label(format!(
                    "{n}. ({} points, {}) {}",
                    q.points.0, q.topic, q.prompt
                ));
            });
            match q.format {
                ExamItemFormat::SingleSelect => {
                    for (idx, opt) in q.options.iter().enumerate() {
                        ui.weak(format!("   ( ) {}. {opt}", choice_tag(idx)));
                    }
                }
                ExamItemFormat::MultipleSelect => {
                    for (idx, opt) in q.options.iter().enumerate() {
                        ui.weak(format!("   [ ] {}. {opt}", choice_tag(idx)));
                    }
                }
                _ => {}
            }
        }
    }
    ui.add_space(8.0);
    if m.show_answers {
        let questions = all_questions(&g.exam);
        ui.label(RichText::new("Reference answers").strong());
        for (i, k) in g.keys.iter().enumerate() {
            let opts = questions.get(i).map_or(&[][..], |q| q.options.as_slice());
            ui.label(format!("{}. {}", i + 1, format_expected_text(k, opts)));
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
