//! The Folders view: Obsidian-like file explorer.

mod dialogs;
mod toolbar;
mod tree;
mod tree_menu;
mod tree_row;

use eframe::egui;

use crate::msg::Msg;
use crate::state::State;

/// Draw the Folders screen: toolbar, tree view, popovers and dialogs.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.vault.is_none() {
        return;
    }

    toolbar::draw(state, ui, out);
    ui.separator();
    tree::draw(state, ui, out);
    dialogs::draw(state, ui, out);
}
