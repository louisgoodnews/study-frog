//! The math drill view (STORY-0184): the problem, its given values as badges, the working rows with
//! a keypad, the check with its feedback, and the full solution on demand.

use eframe::egui::{self, Frame, Stroke, TextEdit};
use studyfrog_rehearsal::formula_trace::{TraceStep, se_trace};

use crate::msg::Msg;
use crate::setup::math::{MathDrill, MathMsg, STEP_LABELS};
use crate::theme::palette_of;
use crate::view::kit;

/// Keys of the keypad, in rows.
const KEYS: [[&str; 4]; 4] = [
    ["7", "8", "9", "Del"],
    ["4", "5", "6", "C"],
    ["1", "2", "3", "-"],
    ["0", ".", "", ""],
];

/// Draw the drill.
pub fn draw(ui: &mut egui::Ui, d: &MathDrill, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    let e = &d.exercise;
    ui.label(format!(
        "A test has standard deviation s_x = {} and reliability r_xx = {}. Compute the standard error of measurement s_e.",
        e.s_x, e.r_xx
    ));
    ui.horizontal_wrapped(|ui| {
        kit::pill(ui, &format!("s_x = {}", e.s_x), pal.accent);
        kit::pill(ui, &format!("r_xx = {}", e.r_xx), pal.accent);
    });
    ui.label(egui::RichText::new("s_e = s_x * sqrt(1 - r_xx)").strong());
    ui.add_space(10.0);
    ui.label("Working (optional), then the answer");
    let trace = se_trace(e.s_x, e.r_xx);
    let wrong = d.feedback.as_ref().and_then(|f| f.diagnosis.deviates_at);
    for i in 0..3 {
        row(ui, d, i, &trace, (wrong == Some(i), d.solution), out);
    }
    ui.add_space(6.0);
    answer_row(ui, d, out);
    keypad(ui, d, out);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if kit::primary_button(ui, "Check")
            .on_hover_text("Checks the working rows you typed, then the answer.")
            .clicked()
        {
            out.push(Msg::Math(MathMsg::Check));
        }
        if kit::quiet_button(ui, "Show full solution")
            .on_hover_text("Shows each step of the solution with its value.")
            .clicked()
        {
            out.push(Msg::Math(MathMsg::Solution));
        }
        if kit::quiet_button(ui, "New problem")
            .on_hover_text("A new problem with other values.")
            .clicked()
        {
            out.push(Msg::Math(MathMsg::Next));
        }
        if kit::quiet_button(ui, "Close drill")
            .on_hover_text("Closes the math drill and returns to the setup.")
            .clicked()
        {
            out.push(Msg::Math(MathMsg::Close));
        }
    });
    feedback(ui, d, &pal);
    if d.solution {
        solution(ui, &trace);
    }
}

fn row(
    ui: &mut egui::Ui,
    d: &MathDrill,
    i: usize,
    trace: &[TraceStep],
    (deviates, solution): (bool, bool),
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    let edge = if deviates { pal.danger } else { pal.border };
    Frame::new()
        .stroke(Stroke::new(if deviates { 2.0 } else { 1.0 }, edge))
        .corner_radius(8)
        .inner_margin(8)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(STEP_LABELS[i]);
                let mut text = d.steps[i].clone();
                let field = ui.add(TextEdit::singleline(&mut text).desired_width(140.0));
                field.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::TextInput,
                        true,
                        format!("Working {}", STEP_LABELS[i]),
                    )
                });
                if field.gained_focus() || field.clicked() {
                    out.push(Msg::Math(MathMsg::Active(i)));
                }
                if field.changed() {
                    out.push(Msg::Math(MathMsg::Step(i, text)));
                }
                // The correct value is shown only with the solution: it would give the answer away.
                if let (true, Some(step)) = (solution, trace.get(i)) {
                    ui.weak(format!("({})", fmt(step.value)));
                }
            });
        });
    ui.add_space(4.0);
}

fn answer_row(ui: &mut egui::Ui, d: &MathDrill, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.label("s_e");
        let mut text = d.answer.clone();
        let field = ui.add(
            TextEdit::singleline(&mut text)
                .hint_text("the answer")
                .desired_width(140.0),
        );
        field.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Answer s_e"));
        if field.changed() {
            out.push(Msg::Math(MathMsg::Answer(text)));
        }
    });
}

fn keypad(ui: &mut egui::Ui, d: &MathDrill, out: &mut Vec<Msg>) {
    ui.add_space(6.0);
    ui.label(format!("Keypad types into: {}", STEP_LABELS[d.active]));
    for r in KEYS {
        ui.horizontal(|ui| {
            for k in r.into_iter().filter(|k| !k.is_empty()) {
                if kit::quiet_button(ui, k)
                    .on_hover_text("Types this key into the row the keypad shows.")
                    .clicked()
                {
                    out.push(Msg::Math(MathMsg::Key(k.to_owned())));
                }
            }
        });
    }
}

fn feedback(ui: &mut egui::Ui, d: &MathDrill, pal: &crate::theme::Palette) {
    let Some(f) = &d.feedback else {
        return;
    };
    ui.add_space(8.0);
    let (color, text) = if f.answer_correct {
        (pal.success, "Correct. Well worked.")
    } else {
        (pal.warning, "Not yet: the answer is outside the tolerance.")
    };
    Frame::new()
        .stroke(Stroke::new(2.0, color))
        .corner_radius(10)
        .inner_margin(10)
        .show(ui, |ui| {
            ui.colored_label(color, text);
            ui.label(&f.diagnosis.message);
        });
}

fn solution(ui: &mut egui::Ui, trace: &[TraceStep]) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Full solution").strong());
    for (label, step) in STEP_LABELS.iter().zip(trace) {
        ui.label(format!("{label} = {}", fmt(step.value)));
    }
}

fn fmt(x: f64) -> String {
    format!("{x:.3}")
}
