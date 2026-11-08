//! "Assign keys…" in the library (STORY-0096): items made before keys existed get theirs only after
//! the user confirms, because it rewrites their files; Undo puts them back.

use eframe::egui;

use crate::item_look::keyless_count;
use crate::library_bulk::BulkMsg;
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: BulkMsg) {
    out.push(Msg::Library(LibraryMsg::Bulk(m)));
}

/// The button, shown while some stacks or items have no key.
pub fn button(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let n = keyless_count(state);
    if n == 0 || state.library.confirm_keys {
        return;
    }
    let hint = format!("{n} stacks and items have no key (such as FLASHCARD_12) yet");
    if ui.button("Assign keys…").on_hover_text(hint).clicked() {
        send(out, BulkMsg::AskAssignKeys(true));
    }
}

/// The question before assigning: how many files change and that it can be undone.
pub fn confirm_bar(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if !state.library.confirm_keys {
        return;
    }
    let n = keyless_count(state);
    ui.horizontal_wrapped(|ui| {
        ui.label(format!(
            "Give the {n} stacks and items without a key their key (STACK_1, FLASHCARD_1, NOTE_1, \
             QUESTION_1, ...)? A `key:` line is added to each file; Undo takes it back."
        ));
        if kit::primary_button(ui, "Assign keys").clicked() {
            send(out, BulkMsg::AssignKeys);
        }
        if ui.button("Cancel").clicked() {
            send(out, BulkMsg::AskAssignKeys(false));
        }
    });
}
