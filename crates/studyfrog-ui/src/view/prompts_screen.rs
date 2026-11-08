//! The Prompts screen: the standard prompts for an AI, on a screen of their own.

use eframe::egui;

use crate::msg::Msg;
use crate::state::State;
use crate::view::{agent_prompts, kit};

/// The screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.heading("Prompts");
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            kit::column(ui, 860.0, |ui| agent_prompts::prompts_tab(state, ui, out));
        });
}
