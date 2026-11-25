//! The Folders view: Obsidian-like file explorer, in the redesign style (EPIC-073, STORY-0233).

mod dialogs;
mod headline;
mod toolbar;
mod tree;
mod tree_menu;
mod tree_row;

use eframe::egui;

use crate::msg::Msg;
use crate::state::State;
use crate::view::home_hero;

/// Draw the Folders screen: the headline, the toolbar and the tree on cards, then the popovers and dialogs.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let _timer = studyfrog_debug::timed("ui", "folders draw");
    studyfrog_log::log_debug!("ui", "folders shown");
    if state.vault.is_none() {
        return;
    }

    headline::draw(state, ui);
    ui.add_space(8.0);
    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        toolbar::draw(state, ui, out);
    });
    ui.add_space(8.0);
    home_hero::card(ui, |ui| {
        tree::draw(state, ui, out);
    });
    dialogs::draw(state, ui, out);
}
