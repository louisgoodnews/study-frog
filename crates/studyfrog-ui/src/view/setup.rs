//! The run setup as three steps: choose what to rehearse, configure how, review and begin. The
//! step bar lets anyone jump to any step; the Next and Back buttons are the guided route.

use eframe::egui::{self, RichText};

use crate::msg::Msg;
use crate::setup::{Preview, SetupStep};
use crate::state::State;
use crate::theme::palette_of;
use crate::view::{kit, setup_how, setup_review, setup_what};

const STEPS: [(SetupStep, &str, &str); 3] = [
    (SetupStep::What, "1  What", "Choose the stacks"),
    (SetupStep::How, "2  How", "Configure your run"),
    (SetupStep::Review, "3  Review", "Check and begin"),
];

fn go(out: &mut Vec<Msg>, step: SetupStep) {
    out.push(Msg::SetupStep(step));
}

/// The whole screen: the steps scroll, the Back and Next buttons stay in view.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let step = state.rehearse.step;
    egui::Panel::bottom("setup-nav")
        .show_separator_line(false)
        .frame(egui::Frame::new().inner_margin(egui::Margin::symmetric(0, 8)))
        .show(ui, |ui| {
            kit::column(ui, kit::READING_WIDTH, |ui| navigation(state, ui, out));
        });
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            kit::column(ui, kit::READING_WIDTH, |ui| {
                ui.label(RichText::new("Rehearse").size(26.0).strong());
                step_bar(step, ui, out);
                ui.add_space(12.0);
                let (title, hint) = match step {
                    SetupStep::What => (
                        "What do you want to rehearse?",
                        "Pick stacks, or leave all unticked for everything.",
                    ),
                    SetupStep::How => (
                        "How do you want to rehearse?",
                        "Start from a preset, then adjust.",
                    ),
                    SetupStep::Review => ("Ready?", "Check your choices, then begin."),
                };
                ui.label(RichText::new(title).size(19.0).strong());
                ui.label(RichText::new(hint).color(palette_of(ui).text_weak));
                match step {
                    SetupStep::What => setup_what::draw(state, ui, out),
                    SetupStep::How => setup_how::draw(state, ui, out),
                    SetupStep::Review => setup_review::draw(state, ui, out),
                }
                ui.add_space(16.0);
            });
        });
}

fn step_bar(current: SetupStep, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        for (step, label, _) in STEPS {
            if ui.selectable_label(step == current, label).clicked() {
                go(out, step);
            }
        }
        let n = STEPS
            .iter()
            .position(|s| s.0 == current)
            .map_or(1, |i| i + 1);
        ui.label(RichText::new(format!("Step {n} of 3")).color(palette_of(ui).text_weak));
    });
}

fn navigation(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let step = state.rehearse.step;
    ui.horizontal(|ui| {
        match step {
            SetupStep::What => {}
            SetupStep::How => {
                if ui.button("Back").clicked() {
                    go(out, SetupStep::What);
                }
            }
            SetupStep::Review => {
                if ui.button("Back").clicked() {
                    go(out, SetupStep::How);
                }
            }
        }
        match step {
            SetupStep::What => {
                if kit::primary_button(ui, "Next: configure").clicked() {
                    go(out, SetupStep::How);
                }
            }
            SetupStep::How => {
                if kit::primary_button(ui, "Next: review").clicked() {
                    go(out, SetupStep::Review);
                }
            }
            SetupStep::Review => start_button(state, ui, out),
        }
        count_hint(&state.rehearse.preview, ui);
    });
}

fn start_button(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let any = matches!(state.rehearse.preview, Preview::Count(n) if n > 0);
    let ready = any && !state.rehearse.starting;
    let label = "Start rehearsal";
    let response = ui.add_enabled(
        ready,
        egui::Button::new(
            RichText::new(label)
                .strong()
                .color(palette_of(ui).on_accent),
        )
        .fill(palette_of(ui).accent),
    );
    if response.clicked() {
        out.push(Msg::StartRunClicked);
    }
    if state.rehearse.starting {
        ui.spinner();
    }
}

/// The live count, shown beside the buttons on every step.
fn count_hint(preview: &Preview, ui: &mut egui::Ui) {
    let weak = palette_of(ui).text_weak;
    match preview {
        Preview::Unknown | Preview::Counting => {
            ui.label(RichText::new("Counting matching items…").color(weak))
        }
        Preview::Count(n) => ui.label(RichText::new(format!("{n} items match")).color(weak)),
        Preview::Invalid(reason) => ui.colored_label(ui.visuals().error_fg_color, reason),
        Preview::Failed(reason) => ui.colored_label(
            ui.visuals().error_fg_color,
            format!("Could not count: {reason}"),
        ),
    };
}
