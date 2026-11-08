//! Zen mode in the window (STORY-0116): its shortcut and its button in the top bar. While it is on, `view::draw`
//! leaves out the sidebar and the status bar; the button (and the shortcut) stays to leave it.

use eframe::egui::{self, Key};

use crate::chrome::ChromeMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::view::icons::{Glyph, icon_button};

/// The name of the button and its shortcut, for screen readers and the tooltip.
pub const NAME: &str = "Zen mode (Ctrl+Shift+F)";

/// Ctrl+Shift+F or F11 turns Zen mode on or off from any screen, also while a text field has the keyboard.
pub fn keys(ui: &egui::Ui, out: &mut Vec<Msg>) {
    let pressed = ui.input(|i| {
        let m = i.modifiers;
        (m.command && m.shift && !m.alt && i.key_pressed(Key::F)) || i.key_pressed(Key::F11)
    });
    if pressed {
        out.push(Msg::Chrome(ChromeMsg::ToggleZen));
    }
}

/// The square button in the top bar; highlighted while Zen mode is on, when it is the way out.
pub fn button(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if icon_button(ui, Glyph::Focus, NAME, crate::zen::active(state)).clicked() {
        out.push(Msg::Chrome(ChromeMsg::ToggleZen));
    }
}
