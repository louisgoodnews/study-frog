//! Ctrl+F and `/` focus the search field of the screen in front (STORY-0188). Each search field asks
//! here whether the shortcut was pressed; only the field on screen is drawn, so only it can take focus.

use eframe::egui::{
    self, Key, TextEdit,
    text::{CCursor, CCursorRange},
};

/// The "Find in this PDF" field's name (its accessible name adds the shortcut, see [`named`]).
pub const PDF_FIND: &str = "Find in this PDF";
/// The library's main search field's accessible name.
pub const LIBRARY_TITLES: &str = "Search titles";
/// The library's query box's accessible name.
pub const LIBRARY_QUERY: &str = "Query text";

/// The accessible name of a search field: its name and the shortcut, so a screen reader says both.
pub fn named(name: &str) -> String {
    format!("{name} (Ctrl+F or /)")
}

/// Whether the shortcut was pressed this frame: Ctrl+F (Cmd+F on macOS), or `/` while no text field has
/// the keyboard.
pub fn pressed(ctx: &egui::Context) -> bool {
    let (ctrl_f, slash) = ctx.input(|i| {
        (
            i.modifiers.command && !i.modifiers.shift && !i.modifiers.alt && i.key_pressed(Key::F),
            i.key_pressed(Key::Slash) && !i.modifiers.any(),
        )
    });
    let typing = ctx.memory(|m| m.focused().is_some());
    ctrl_f || (slash && !typing)
}

/// Give the field the keyboard and select its `len` characters when the shortcut was pressed, so the
/// next key replaces the text. Call it with the field's response, right after the field is drawn.
pub fn focus_field(ctx: &egui::Context, response: &egui::Response, len: usize) {
    if !pressed(ctx) {
        return;
    }
    response.request_focus();
    if let Some(mut st) = TextEdit::load_state(ctx, response.id) {
        st.cursor
            .set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(len))));
        TextEdit::store_state(ctx, response.id, st);
    }
}
