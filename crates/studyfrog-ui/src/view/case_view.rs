//! The case-study view (STORY-0181): the vignette in a framed block, the decision with its options, the
//! justification, and the evaluation: a status pill, the choice, and a met or missing pill per rubric
//! point with its advice.

use eframe::egui::{self, Color32, Frame, Stroke, TextEdit};
use studyfrog_agent::rubric_eval::EvalReport;

use crate::msg::Msg;
use crate::setup::case::{CaseMsg, CaseUi};
use crate::theme::palette_of;
use crate::view::kit;

/// Draw the case-study view for the open case.
pub fn draw(ui: &mut egui::Ui, cs: &CaseUi, out: &mut Vec<Msg>) {
    if cs.loading {
        ui.label("Reading the case studies…");
    }
    if let Some(error) = &cs.error {
        let pal = palette_of(ui);
        ui.colored_label(pal.danger, error);
    }
    let Some((_, case)) = cs.cases.get(cs.current) else {
        close(ui, out);
        return;
    };
    ui.label(format!("Case {} of {}", cs.current + 1, cs.cases.len()));
    vignette(ui, &case.vignette);
    ui.add_space(12.0);
    ui.label(egui::RichText::new(&case.decision_prompt).strong());
    for (i, option) in case.options.iter().enumerate() {
        let mut on = cs.picked.get(i).copied().unwrap_or(false);
        if ui.checkbox(&mut on, option).changed() {
            out.push(Msg::Case(CaseMsg::Pick(i, on)));
        }
    }
    ui.add_space(12.0);
    ui.label("Your justification");
    let mut text = cs.rationale.clone();
    let field = ui.add(
        TextEdit::multiline(&mut text)
            .desired_rows(6)
            .desired_width(f32::INFINITY)
            .hint_text("Why this choice? Name the points the case turns on."),
    );
    field.widget_info(|| {
        egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Your justification")
    });
    if field.changed() {
        out.push(Msg::Case(CaseMsg::Rationale(text)));
    }
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if kit::primary_button(ui, "Check my answer")
            .on_hover_text("Scores the choice and the justification against the case's rubric.")
            .clicked()
        {
            out.push(Msg::Case(CaseMsg::Submit));
        }
        if cs.cases.len() > 1
            && kit::quiet_button(ui, "Next case")
                .on_hover_text("Goes on to the next case of this stack.")
                .clicked()
        {
            out.push(Msg::Case(CaseMsg::Next));
        }
        close(ui, out);
    });
    if let Some(report) = &cs.report {
        ui.add_space(12.0);
        report_view(ui, report, &case.rubric_criteria);
    }
}

fn close(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if kit::quiet_button(ui, "Close case studies")
        .on_hover_text("Closes the case studies and returns to the setup.")
        .clicked()
    {
        out.push(Msg::Case(CaseMsg::Close));
    }
}

fn vignette(ui: &mut egui::Ui, text: &str) {
    let pal = palette_of(ui);
    Frame::new()
        .fill(pal.surface)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(10)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            ui.label(text);
        });
}

/// The status of an answer and its colour, from the choice and the overall score.
fn verdict(report: &EvalReport, pal: &crate::theme::Palette) -> (&'static str, Color32) {
    if report.choice_valid && report.overall_pct >= 80.0 {
        ("Solid decision", pal.success)
    } else if report.overall_pct >= 50.0 {
        ("Partial match", pal.warning)
    } else if !report.choice_valid {
        ("Alternative procedure recommended", pal.danger)
    } else {
        ("Needs more justification", pal.danger)
    }
}

fn report_view(ui: &mut egui::Ui, report: &EvalReport, rubric: &[String]) {
    let pal = palette_of(ui);
    let (status, color) = verdict(report, &pal);
    ui.horizontal_wrapped(|ui| {
        kit::pill(ui, status, color);
        kit::pill(ui, &format!("{:.0} %", report.overall_pct), pal.accent);
        if report.choice_valid {
            kit::pill(ui, "Choice: the expected one", pal.success);
        } else {
            kit::pill(ui, "Choice: not the expected one", pal.danger);
        }
    });
    for (criterion, result) in rubric.iter().zip(&report.criteria) {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if result.met {
                kit::pill(ui, "Met", pal.success);
            } else {
                kit::pill(ui, "Missing", pal.danger);
            }
            ui.label(criterion);
        });
        if !result.feedback.is_empty() {
            ui.label(egui::RichText::new(&result.feedback).color(pal.text_weak));
        }
    }
}
