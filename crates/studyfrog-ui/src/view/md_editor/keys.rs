//! Keys in a Markdown text area that do more than type: Enter and Backspace in a list. The key is taken from egui
//! only when the edit is a list edit, so everywhere else Enter and Backspace work as they always did.

use eframe::egui::{self, Id, Key, Modifiers, TextEdit};

use crate::md_edit::Edited;
use crate::md_lists::{on_backspace, on_enter};

/// A list edit: the text and the cursor in, the new text and cursor out when the key makes an edit.
type ListEdit = fn(&str, usize) -> Option<Edited>;

/// The edit a list key makes in the focused text area `text_id` holding `text`, if it makes one.
pub(super) fn list_key(ui: &egui::Ui, text_id: Id, text: &str) -> Option<Edited> {
    if !ui.memory(|m| m.has_focus(text_id)) {
        return None;
    }
    let pressed = |key| ui.input(|i| i.key_pressed(key) && i.modifiers.is_none());
    let (key, edit): (Key, ListEdit) = if pressed(Key::Enter) {
        (Key::Enter, on_enter)
    } else if pressed(Key::Backspace) {
        (Key::Backspace, on_backspace)
    } else {
        return None;
    };
    let range = TextEdit::load_state(ui.ctx(), text_id)?
        .cursor
        .char_range()?;
    if range.primary != range.secondary {
        return None; // a selection: the keys replace it as usual
    }
    let edited = edit(text, range.primary.index.into())?;
    ui.input_mut(|i| i.consume_key(Modifiers::NONE, key));
    Some(edited)
}

/// Ctrl+Shift+C wraps the selection of the focused text area in a cloze gap with the next number,
/// Ctrl+Shift+Alt+C with the number used last (STORY-0108).
pub(super) fn cloze_key(ui: &egui::Ui, text_id: Id, text: &str) -> Option<Edited> {
    if !ui.memory(|m| m.has_focus(text_id)) {
        return None;
    }
    let same = Modifiers::COMMAND | Modifiers::SHIFT | Modifiers::ALT;
    let next = Modifiers::COMMAND | Modifiers::SHIFT;
    let pressed =
        |m: Modifiers| ui.input(|i| i.modifiers.matches_exact(m) && i.key_pressed(Key::C));
    let (mods, keep) = if pressed(same) {
        (same, true)
    } else if pressed(next) {
        (next, false)
    } else {
        return None;
    };
    let range = TextEdit::load_state(ui.ctx(), text_id)?
        .cursor
        .char_range()?;
    let sel = (range.primary.index.into(), range.secondary.index.into());
    ui.input_mut(|i| i.consume_key(mods, Key::C));
    Some(crate::md_edit::apply_cmd(
        text,
        sel,
        crate::md_edit::Cmd::Cloze(keep),
    ))
}
