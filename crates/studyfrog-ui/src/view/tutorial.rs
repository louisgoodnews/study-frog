//! The tutorials: their entry in the help screen's tree and the card that guides through the steps.

use eframe::egui::{self, Align2, Order, RichText, vec2};

use crate::help_types::HelpMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::tutorial::{FIRST_STEPS, TutorialMsg, current};

fn send(out: &mut Vec<Msg>, m: TutorialMsg) {
    out.push(Msg::Help(HelpMsg::Tutorial(m)));
}

/// The tutorial section of the help screen's tree.
pub fn tutorials(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(RichText::new("Tutorials").strong());
    if state.vault.is_none() {
        ui.weak("Open a vault to take a tutorial.");
        return;
    }
    let label = if state.help.tutorial.is_some() {
        "Restart \"First steps\""
    } else {
        "Start \"First steps\""
    };
    if ui
        .button(label)
        .on_hover_text("A stack, a card, a rehearsal and the calendar, one step at a time.")
        .clicked()
    {
        send(out, TutorialMsg::Start);
    }
}

/// The card of the step under way, at the lower right of every screen.
pub fn panel(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some((t, step)) = current(state) else {
        return;
    };
    egui::Area::new(egui::Id::unique("tutorial-panel"))
        .order(Order::Foreground)
        .anchor(Align2::RIGHT_BOTTOM, vec2(-16.0, -40.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(320.0);
                let n = FIRST_STEPS.len();
                ui.weak(format!("First steps · step {} of {n}", t.step + 1));
                ui.label(RichText::new(step.title).strong());
                ui.label(step.text);
                if t.done {
                    ui.label(RichText::new("Done!").strong());
                }
                buttons(t.done, t.step + 1 == n, ui, out);
            });
        });
}

fn buttons(done: bool, last: bool, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        let next = if last { "Finish" } else { "Next step" };
        if done && ui.button(next).clicked() {
            send(out, TutorialMsg::Next);
        }
        if !done && ui.button("Show me where").clicked() {
            send(out, TutorialMsg::Show);
        }
        if ui.button("End tutorial").clicked() {
            send(out, TutorialMsg::Stop);
        }
    });
}
