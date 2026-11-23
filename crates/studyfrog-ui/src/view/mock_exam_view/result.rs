//! The graded exam (STORY-0187): the grade and the total, the points by topic, and each question with the
//! student's answer, the model answer, and the rubric points made.

use eframe::egui::{self, RichText};
use studyfrog_agent::rubric_eval::KEYWORD_CAVEAT;
use studyfrog_rehearsal::grade::{grade_label, is_passing};

use crate::setup::mock_exam::grading::{Graded, breakdown, total};
use crate::theme::palette_of;

/// Who graded the answers, in one line.
pub fn engine_line(graded: &[Graded]) -> String {
    let model = graded.iter().filter(|g| g.from_model).count();
    match (model, graded.len()) {
        (_, 0) => String::new(),
        (n, all) if n == all => "Graded by the local model.".to_owned(),
        (0, _) => "Graded by word overlap: no local model is selected, so a correct answer in other words can be marked missing.".to_owned(),
        (n, all) => format!("{n} of {all} answers graded by the local model; the rest by word overlap."),
    }
}

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
    ui.weak(engine_line(graded));
    let left_out = graded.iter().filter(|g| !g.counted).count();
    if left_out > 0 {
        ui.weak(format!(
            "{left_out} question(s) not counted: no rubric point of them could be checked by word overlap."
        ));
    }
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
        let score = if g.counted {
            format!("{} / {} points", g.earned, g.possible)
        } else {
            "not counted".to_owned()
        };
        ui.label(RichText::new(format!("{}. ({score}, {}) {}", i + 1, g.topic, g.prompt)).strong());
        ui.label(format!(
            "Your answer: {}",
            if g.answer.trim().is_empty() {
                "(none)"
            } else {
                g.answer.trim()
            }
        ));
        ui.label(format!("Model answer: {}", g.sample_answer));
        if let Some(caveat) = g.caveat.as_deref().filter(|c| *c != KEYWORD_CAVEAT) {
            ui.weak(caveat);
        }
        for (point, met) in &g.points_made {
            ui.horizontal_wrapped(|ui| {
                let (word, c) = match met {
                    Some(true) => ("Made", pal.success),
                    Some(false) => ("Missing", pal.danger),
                    None => ("Not checked by word overlap", pal.text_weak),
                };
                crate::view::kit::pill(ui, word, c);
                ui.label(point);
            });
        }
    }
}
