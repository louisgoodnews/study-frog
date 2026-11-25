//! The Prompts screen: the standard prompts for an AI, on a screen of their own.

use eframe::egui;

use crate::msg::Msg;
use crate::state::State;
use crate::view::{agent_prompts, kit, prompts_header};

/// The screen: the headline, then the prompts on cards.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    prompts_header::headline(ui);
    ui.add_space(8.0);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            kit::column(ui, 860.0, |ui| agent_prompts::prompts_tab(state, ui, out));
        });
}
