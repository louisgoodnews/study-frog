//! The history buttons of the top bar: Back, Forward and the list of places visited.

use eframe::egui::{self, Rect};

use crate::chrome::{ChromeMsg, Menu};
use crate::msg::Msg;
use crate::state::State;
use crate::view::icons::{Glyph, icon_button};

fn toggle(out: &mut Vec<Msg>, menu: Menu) {
    out.push(Msg::Chrome(ChromeMsg::ToggleMenu(menu)));
}

/// Back, Forward and the list of places visited, like a browser's.
pub(super) fn buttons(
    state: &State,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    anchors: &mut Vec<(Menu, Rect)>,
) {
    let h = &state.history;
    let go = |m| Msg::Chrome(ChromeMsg::History(m));
    let (back, forward) = (crate::history::can_back(h), crate::history::can_forward(h));
    let hint = "Back (Alt+Left)";
    if ui
        .add_enabled_ui(back, |ui| icon_button(ui, Glyph::Back, hint, false))
        .inner
        .clicked()
    {
        out.push(go(crate::history::HistoryMsg::Back));
    }
    let hint = "Forward (Alt+Right)";
    if ui
        .add_enabled_ui(forward, |ui| icon_button(ui, Glyph::Forward, hint, false))
        .inner
        .clicked()
    {
        out.push(go(crate::history::HistoryMsg::Forward));
    }
    let list = icon_button(
        ui,
        Glyph::History,
        "History",
        state.chrome.menu == Some(Menu::History),
    );
    if list.clicked() {
        toggle(out, Menu::History);
    }
    anchors.push((Menu::History, list.rect));
}
