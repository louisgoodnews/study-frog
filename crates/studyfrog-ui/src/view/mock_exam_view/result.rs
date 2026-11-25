//! The graded exam (STORY-0187, STORY-0207): totals, format/topic breakdowns, explicit multi-select rule,
//! and per-question review across all 6 item formats.

use eframe::egui::{self, RichText};
use studyfrog_agent::rubric_eval::KEYWORD_CAVEAT;
use studyfrog_rehearsal::exam_answer::MULTI_SELECT_RULE;
use studyfrog_rehearsal::exam_format::{ExamItemFormat, format_label};
use studyfrog_rehearsal::grade::{grade_label, is_passing};

use crate::msg::Msg;
use crate::setup::mock_exam::MockExamMsg;
use crate::setup::mock_exam::grading::{
    Graded, GradingEngine, breakdown, breakdown_by_format, total,
};
use crate::theme::palette_of;
use crate::view::{home_hero, kit};

/// Summary sentence describing which grading engines scored the exam.
pub fn engine_line(graded: &[Graded]) -> String {
    if graded.is_empty() {
        return String::new();
    }
    let objective = graded
        .iter()
        .filter(|g| g.engine == GradingEngine::Objective)
        .count();
    let model = graded
        .iter()
        .filter(|g| g.engine == GradingEngine::Model)
        .count();
    let overlap = graded
        .iter()
        .filter(|g| g.engine == GradingEngine::WordOverlap)
        .count();
    let mut parts = Vec::new();
    if objective > 0 {
        parts.push(format!("{objective} objective"));
    }
    if model > 0 {
        parts.push(format!("{model} by local model"));
    }
    if overlap > 0 {
        parts.push(format!(
            "{overlap} by word overlap (no local model selected, so phrasing differences can be marked missing)"
        ));
    }
    format!("Graded: {}.", parts.join("; "))
}

fn pass_mark_line(ui: &mut egui::Ui, pass_mark: f32, out: &mut Vec<Msg>) {
    let mut pct = pass_mark;
    ui.horizontal(|ui| {
        ui.label("Pass mark");
        let field = ui
            .add(
                egui::DragValue::new(&mut pct)
                    .range(1.0..=100.0)
                    .speed(1.0)
                    .max_decimals(0)
                    .suffix(" %"),
            )
            .on_hover_text("The share of the points needed to pass. Below it the grade is 5.0.");
        if field.changed() {
            out.push(Msg::MockExam(MockExamMsg::SetPassMark(pct)));
        }
    });
}

fn engine_badge(engine: GradingEngine) -> &'static str {
    match engine {
        GradingEngine::Objective => "Objective",
        GradingEngine::Model => "Local model",
        GradingEngine::WordOverlap => "Word overlap",
    }
}

/// Draw the result of a graded exam at its pass mark.
pub fn draw(ui: &mut egui::Ui, graded: &[Graded], pass_mark: f32, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    let t = total(graded, pass_mark);
    let pct = t.percent.unwrap_or(0.0);
    let (verdict, color) = if is_passing(t.grade) {
        ("Passed", pal.success)
    } else {
        ("Not passed", pal.danger)
    };
    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        pass_mark_line(ui, pass_mark, out);
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
        if graded
            .iter()
            .any(|g| g.format == ExamItemFormat::MultipleSelect)
        {
            ui.weak(MULTI_SELECT_RULE);
        }
        let left_out = graded.iter().filter(|g| !g.counted).count();
        if left_out > 0 {
            ui.weak(format!(
                "{left_out} question(s) not counted: no rubric point of them could be checked by word overlap."
            ));
        }
    });
    draw_breakdowns(ui, graded, pass_mark);
    ui.add_space(8.0);
    ui.label(RichText::new("Review").strong());
    for (i, g) in graded.iter().enumerate() {
        draw_question_review(ui, i, g);
    }
}

fn draw_breakdowns(ui: &mut egui::Ui, graded: &[Graded], pass_mark: f32) {
    let pal = palette_of(ui);
    ui.add_space(8.0);
    ui.label(RichText::new("By format").strong());
    for (fmt, earned, possible) in breakdown_by_format(graded) {
        let share = if possible == 0 {
            0.0
        } else {
            earned as f32 / possible as f32
        };
        let c = if share * 100.0 >= pass_mark {
            pal.success
        } else {
            pal.danger
        };
        ui.horizontal(|ui| {
            ui.label(format!("{}: {earned} / {possible}", format_label(fmt)));
            ui.colored_label(c, format!("{:.0} %", share * 100.0));
        });
    }
    ui.add_space(6.0);
    ui.label(RichText::new("By topic").strong());
    for (topic, earned, possible) in breakdown(graded) {
        let share = if possible == 0 {
            0.0
        } else {
            earned as f32 / possible as f32
        };
        let c = if share * 100.0 >= pass_mark {
            pal.success
        } else {
            pal.danger
        };
        ui.horizontal(|ui| {
            ui.label(format!("{topic}: {earned} / {possible}"));
            ui.colored_label(c, format!("{:.0} %", share * 100.0));
        });
    }
}

fn draw_question_review(ui: &mut egui::Ui, i: usize, g: &Graded) {
    ui.add_space(6.0);
    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        question_review(ui, i, g);
    });
}

fn question_review(ui: &mut egui::Ui, i: usize, g: &Graded) {
    let pal = palette_of(ui);
    let score = if g.counted {
        format!("{} / {} points", g.earned, g.possible)
    } else {
        "not counted".to_owned()
    };
    ui.horizontal_wrapped(|ui| {
        kit::pill(ui, format_label(g.format), pal.accent);
        kit::pill(ui, engine_badge(g.engine), pal.text_weak);
        ui.label(RichText::new(format!("{}. ({score}, {}) {}", i + 1, g.topic, g.prompt)).strong());
    });
    ui.label(format!(
        "Your answer: {}",
        if g.answer.trim().is_empty() {
            "(none)"
        } else {
            g.answer.trim()
        }
    ));
    ui.label(format!("Expected answer: {}", g.sample_answer));
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
            kit::pill(ui, word, c);
            ui.label(point);
        });
    }
}
