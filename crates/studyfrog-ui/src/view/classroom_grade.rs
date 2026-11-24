//! The teacher's grade of one answer (TASK-0294): the score, the letter, each rubric point's verdict
//! with its sentence, and the narrative feedback.

use eframe::egui::{self, RichText};
use studyfrog_agent::lecture_grade::label_of;
use studyfrog_core::classroom::{CriterionMark, GradeReport};

use crate::theme::{palette_of, tint};

/// Draws the grade. Points are shown out of the grade's own maximum, the letter from the share of it.
pub fn draw(ui: &mut egui::Ui, grade: &GradeReport) {
    let pal = palette_of(ui);
    let share = grade.points.saturating_mul(100) / grade.max.max(1);
    ui.label(RichText::new("Teacher's grade").strong());
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(format!("Score: {} of {}", grade.points, grade.max)).strong());
        ui.label(
            RichText::new(format!("Grade: {}", label_of(share)))
                .strong()
                .color(pal.accent),
        );
    });
    for mark in &grade.criteria {
        criterion(ui, mark);
    }
    if !grade.feedback.trim().is_empty() {
        ui.add_space(4.0);
        ui.label(RichText::new("Feedback").small().strong());
        ui.add(egui::Label::new(grade.feedback.trim()).wrap());
    }
}

/// One rubric point: its verdict badge, the point itself and the teacher's sentence about it.
fn criterion(ui: &mut egui::Ui, mark: &CriterionMark) {
    let pal = palette_of(ui);
    let (verdict, color) = match (mark.checked, mark.met) {
        (false, _) => ("Not checked", pal.text_weak),
        (true, true) => ("Met", pal.success),
        (true, false) => ("Missing", pal.warning),
    };
    ui.horizontal_wrapped(|ui| {
        let tag = egui::Frame::new()
            .fill(tint(color, 40))
            .corner_radius(4)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ui.label(RichText::new(verdict).small().strong().color(color))
            })
            .inner;
        let access = format!("Rubric point {verdict}");
        tag.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &access));
        ui.label(&mark.criterion);
    });
    if !mark.feedback.trim().is_empty() {
        ui.weak(mark.feedback.trim());
    }
    ui.add_space(4.0);
}
