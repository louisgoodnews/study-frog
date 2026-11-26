//! The Folders view: Nautilus/Thunar-style vault file manager (EPIC-074).

mod dialogs;
mod grid;
mod headline;
mod icons;
mod list;
mod status;
mod toolbar;
mod tree;
mod tree_menu;
mod tree_row;

use eframe::egui;

use crate::folders_browse::FoldersViewMode;
use crate::msg::Msg;
use crate::state::State;
use crate::view::home_hero;

/// Draw the Folders screen: headline, Nautilus header toolbar, active view (Grid/List/Tree),
/// selection status pill, and dialogs.
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
        ui.set_width(ui.available_width());
        match state.folders.view_mode {
            FoldersViewMode::Grid => grid::draw(state, ui, out),
            FoldersViewMode::List => list::draw(state, ui, out),
            FoldersViewMode::Tree => tree::draw(state, ui, out),
        }
    });
    ui.add_space(6.0);
    status::draw(state, ui);
    dialogs::draw(state, ui, out);
}
