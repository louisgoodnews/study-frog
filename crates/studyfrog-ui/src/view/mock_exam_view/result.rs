//! The graded exam (STORY-0187): the grade and the total, the points by topic, and each question with the
//! student's answer, the model answer, and the rubric points made.

use eframe::egui::{self, RichText};
use studyfrog_rehearsal::grade::{grade_label, is_passing};

use crate::setup::mock_exam::grading::{Graded, breakdown, total};
use crate::theme::palette_of;

/// Draw the result of a graded exam.
pub fn draw(ui: &mut egui::Ui, graded: &[Graded]) {
    let pal = palette_of(ui);
    let t = total(graded);
    let pct = t.percent.unwrap_or(0.0);
    let (verdict, color) = if is_passing(t.grade) {
        ("Passed", pal.success)
    } else {
        ("Not passed", pal.danger)
    };
    ui.label(
        RichText::new(format!(
            "{} / {} points · {:.0} % · Grade {} ({})",
            t.earned,
            t.possible,
            pct,
            grade_label(t.grade),
            t.word
        ))
        .strong(),
    );
    ui.colored_label(color, verdict);
    ui.add_space(8.0);
    ui.label(RichText::new("By topic").strong());
    for (topic, earned, possible) in breakdown(graded) {
        let share = if possible == 0 {
            0.0
        } else {
            earned as f32 / possible as f32
        };
        let c = if share * 100.0 >= crate::setup::mock_exam::grading::PASS_MARK {
            pal.success
        } else {
            pal.danger
        };
        ui.horizontal(|ui| {
            ui.label(format!("{topic}: {earned} / {possible}"));
            ui.colored_label(c, format!("{:.0} %", share * 100.0));
        });
    }
    ui.add_space(8.0);
    ui.label(RichText::new("Review").strong());
    for (i, g) in graded.iter().enumerate() {
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!(
                "{}. ({} / {} points, {}) {}",
                i + 1,
                g.earned,
                g.possible,
                g.topic,
                g.prompt
            ))
            .strong(),
        );
        ui.label(format!(
            "Your answer: {}",
            if g.answer.trim().is_empty() {
                "(none)"
            } else {
                g.answer.trim()
            }
        ));
        ui.label(format!("Model answer: {}", g.sample_answer));
        for (point, met) in &g.points_made {
            ui.horizontal_wrapped(|ui| {
                let (word, c) = if *met {
                    ("Made", pal.success)
                } else {
                    ("Missing", pal.danger)
                };
                crate::view::kit::pill(ui, word, c);
                ui.label(point);
            });
        }
    }
}
