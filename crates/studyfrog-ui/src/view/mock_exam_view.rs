//! The mock exam view (STORY-0186, STORY-0187, STORY-0207, STORY-0241): slide picker, format-aware preview,
//! answering, graded review, and saving as a note. The title sits on the redesign's gradient panel.

mod answering;
mod preview;
mod result;

use eframe::egui::{self, RichText};

use crate::msg::Msg;
use crate::setup::mock_exam::{MockExamMsg, MockExamUi, SlideChoice};
use crate::theme::palette_of;
use crate::view::{gradient_panel, kit};

/// The height of the title's gradient panel, in points.
const HEADER_HEIGHT: f32 = 64.0;

/// Draw the exam view.
pub fn draw(ui: &mut egui::Ui, m: &MockExamUi, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    headline(ui, &m.title);
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
            preview::draw(ui, g, m, out);
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

/// The exam's title on the gradient panel.
fn headline(ui: &mut egui::Ui, title: &str) {
    gradient_panel::panel(ui, HEADER_HEIGHT, |ui, s| {
        ui.label(
            RichText::new(title)
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong()
                .color(s.gradient_text),
        );
    });
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
