//! Format-specific mock exam answering controls (STORY-0207).

use eframe::egui::{self, RichText, Role, TextEdit, WidgetInfo};
use studyfrog_rehearsal::exam_answer::{ExamAnswer, choice_tag};
use studyfrog_rehearsal::exam_format::{ExamItemFormat, format_label};
use studyfrog_rehearsal::mock_exam::{ExamQuestion, all_questions};
use studyfrog_rehearsal::mock_timer::{TimerStatus, timer_status};

use crate::msg::Msg;
use crate::run_clock::mmss;
use crate::setup::mock_exam::{MockExamMsg, MockExamUi};
use crate::theme::palette_of;
use crate::view::{home_hero, kit};

fn timer_line(ui: &mut egui::Ui, limit_sec: u32, elapsed_sec: u32) {
    let pal = palette_of(ui);
    let left = limit_sec.saturating_sub(elapsed_sec);
    ui.label(RichText::new(format!("Time left {}", mmss(left * 1000))).strong());
    match timer_status(limit_sec, elapsed_sec) {
        TimerStatus::TwoMinutesLeft => {
            ui.colored_label(pal.danger, "Two minutes left");
        }
        TimerStatus::TenMinutesLeft => {
            ui.colored_label(pal.danger, "Ten minutes left");
        }
        TimerStatus::Running { .. } | TimerStatus::Expired => {}
    }
}

/// Draw the active mock exam answering form across all 6 item formats.
pub fn draw(
    ui: &mut egui::Ui,
    g: &studyfrog_service::exam_gen::Generated,
    m: &MockExamUi,
    out: &mut Vec<Msg>,
) {
    // The timer and the finish button sit together above the questions, so the exam can be submitted without
    // scrolling past every question (the redesign cards made the form taller than a 1000 by 720 window).
    ui.horizontal_wrapped(|ui| {
        if m.locked {
            ui.weak("Submitted. Your answers are locked until the result is shown.");
        } else {
            timer_line(ui, g.exam.time_limit_min * 60, m.elapsed_sec);
            finish_button(ui, out);
        }
    });
    ui.add_enabled_ui(!m.locked, |ui| {
        for (i, q) in all_questions(&g.exam).into_iter().enumerate() {
            ui.add_space(6.0);
            home_hero::card(ui, |ui| {
                ui.set_width(ui.available_width());
                question(ui, i, q, m.answers.get(i), out);
            });
        }
    });
}

fn finish_button(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if kit::primary_button(ui, "Finish and grade")
        .on_hover_text(
            "Grades every answer and shows the grade with the review. The exam also ends when the time runs out.",
        )
        .clicked()
    {
        out.push(Msg::MockExam(MockExamMsg::Finish));
    }
}

/// One question's heading and its answer control, chosen by the item's format.
fn question(
    ui: &mut egui::Ui,
    i: usize,
    q: &ExamQuestion,
    ans: Option<&ExamAnswer>,
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    ui.horizontal_wrapped(|ui| {
        kit::pill(ui, format_label(q.format), pal.accent);
        ui.label(format!("{}. ({} points, {})", i + 1, q.points.0, q.topic));
    });
    match q.format {
        ExamItemFormat::OpenQuestion
        | ExamItemFormat::Calculation
        | ExamItemFormat::CaseScenario => draw_text(ui, i, q, ans, out),
        ExamItemFormat::FillInGaps => draw_gaps(ui, i, q, ans, out),
        ExamItemFormat::SingleSelect => draw_single(ui, i, q, ans, out),
        ExamItemFormat::MultipleSelect => draw_multiple(ui, i, q, ans, out),
    }
}

fn draw_text(
    ui: &mut egui::Ui,
    q_idx: usize,
    q: &ExamQuestion,
    ans: Option<&ExamAnswer>,
    out: &mut Vec<Msg>,
) {
    ui.label(&q.prompt);
    let mut text = match ans {
        Some(ExamAnswer::Text(s)) => s.clone(),
        _ => String::new(),
    };
    let field = ui.add(
        TextEdit::multiline(&mut text)
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );
    let label = format!("Answer {}: {}", q_idx + 1, format_label(q.format));
    field.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, &label));
    if field.changed() {
        out.push(Msg::MockExam(MockExamMsg::AnswerText(q_idx, text)));
    }
}

fn draw_gaps(
    ui: &mut egui::Ui,
    q_idx: usize,
    q: &ExamQuestion,
    ans: Option<&ExamAnswer>,
    out: &mut Vec<Msg>,
) {
    let gaps = match ans {
        Some(ExamAnswer::Gaps(v)) => v.as_slice(),
        _ => &[],
    };
    let mut rest = q.prompt.as_str();
    let mut rendered = vec![false; q.gap_hints.len()];
    ui.horizontal_wrapped(|ui| {
        for (g_idx, done) in rendered.iter_mut().enumerate() {
            let marker = format!("[{}]", g_idx + 1);
            if let Some(pos) = rest.find(&marker) {
                let before = &rest[..pos];
                if !before.is_empty() {
                    ui.label(before);
                }
                gap_input(ui, q_idx, g_idx, q.gap_hints[g_idx].as_deref(), gaps, out);
                *done = true;
                rest = &rest[pos + marker.len()..];
            }
        }
        if !rest.is_empty() {
            ui.label(rest);
        }
    });
    for (g_idx, done) in rendered.into_iter().enumerate() {
        if !done {
            gap_input(ui, q_idx, g_idx, q.gap_hints[g_idx].as_deref(), gaps, out);
        }
    }
}

fn gap_input(
    ui: &mut egui::Ui,
    q_idx: usize,
    g_idx: usize,
    hint: Option<&str>,
    gaps: &[String],
    out: &mut Vec<Msg>,
) {
    let mut val = gaps.get(g_idx).cloned().unwrap_or_default();
    let placeholder = hint.unwrap_or("...");
    let field = ui.add(
        TextEdit::singleline(&mut val)
            .hint_text(placeholder)
            .desired_width(130.0),
    );
    let label = format!("Question {} gap {}", q_idx + 1, g_idx + 1);
    field.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, &label));
    if field.changed() {
        out.push(Msg::MockExam(MockExamMsg::AnswerGap(q_idx, g_idx, val)));
    }
}

fn draw_single(
    ui: &mut egui::Ui,
    q_idx: usize,
    q: &ExamQuestion,
    ans: Option<&ExamAnswer>,
    out: &mut Vec<Msg>,
) {
    ui.label(&q.prompt);
    let selected = match ans {
        Some(ExamAnswer::Single(s)) => *s,
        _ => None,
    };
    for (opt_idx, opt) in q.options.iter().enumerate() {
        let tag = choice_tag(opt_idx);
        let r = ui.radio(selected == Some(opt_idx), format!("{tag}. {opt}"));
        let label = format!("Question {}: {tag}. {opt}", q_idx + 1);
        r.widget_info(|| WidgetInfo::labeled(Role::RadioButton, true, &label));
        if r.clicked() {
            out.push(Msg::MockExam(MockExamMsg::AnswerSingle(q_idx, opt_idx)));
        }
    }
}

fn draw_multiple(
    ui: &mut egui::Ui,
    q_idx: usize,
    q: &ExamQuestion,
    ans: Option<&ExamAnswer>,
    out: &mut Vec<Msg>,
) {
    ui.label(&q.prompt);
    ui.weak("Select all that apply.");
    let flags = match ans {
        Some(ExamAnswer::Multiple(v)) => v.as_slice(),
        _ => &[],
    };
    for (opt_idx, opt) in q.options.iter().enumerate() {
        let tag = choice_tag(opt_idx);
        let mut on = flags.get(opt_idx).copied().unwrap_or(false);
        let r = ui.checkbox(&mut on, format!("{tag}. {opt}"));
        let label = format!("Question {}: {tag}. {opt}", q_idx + 1);
        r.widget_info(|| WidgetInfo::labeled(Role::CheckBox, true, &label));
        if r.changed() {
            out.push(Msg::MockExam(MockExamMsg::AnswerMultiple(
                q_idx, opt_idx, on,
            )));
        }
    }
}
