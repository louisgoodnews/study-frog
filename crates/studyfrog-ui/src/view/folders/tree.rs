//! Tree rendering and keyboard handling for Folders view.

use eframe::egui::{self, Key, ScrollArea};

use crate::folders_tree::{entries_from_vault, tree_visible_rows};
use crate::folders_types::{FoldersKey, FoldersMsg, FoldersState, VisibleRow};
use crate::msg::Msg;
use crate::state::State;
use crate::view::folders::tree_row;

fn check_keys(ui: &egui::Ui, out: &mut Vec<Msg>) {
    let pressed = |k| ui.input(|i| i.key_pressed(k));
    if pressed(Key::ArrowUp) {
        out.push(Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Up)));
    } else if pressed(Key::ArrowDown) {
        out.push(Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Down)));
    } else if pressed(Key::ArrowLeft) {
        out.push(Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Left)));
    } else if pressed(Key::ArrowRight) {
        out.push(Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Right)));
    } else if pressed(Key::Enter) {
        out.push(Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Enter)));
    } else if pressed(Key::F2) {
        out.push(Msg::Folders(FoldersMsg::KeyAction(FoldersKey::F2)));
    } else if pressed(Key::Delete) || pressed(Key::Backspace) {
        out.push(Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Delete)));
    }
}

fn visible_rows(state: &State) -> Vec<VisibleRow> {
    if state.folders.entries.is_empty()
        && let Some(v) = &state.vault
    {
        let entries = entries_from_vault(v, &state.library.items);
        let temp = FoldersState {
            entries,
            expanded: state.folders.expanded.clone(),
            sort: state.folders.sort,
            ..Default::default()
        };
        return tree_visible_rows(&temp);
    }
    tree_visible_rows(&state.folders)
}

/// Draw the virtualized tree view.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    check_keys(ui, out);

    let rows = visible_rows(state);
    let row_height = 24.0;

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, row_height, rows.len(), |ui, row_range| {
            for idx in row_range {
                if let Some(row) = rows.get(idx) {
                    tree_row::draw_row(state, row, ui, out);
                }
            }
        });
}
