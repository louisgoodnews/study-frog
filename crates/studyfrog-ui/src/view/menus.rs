//! What the top-bar menus contain, and the search results under the search field.

use eframe::egui::{self, Rect};

use crate::chrome::{ChromeMsg, Menu};
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::transfer_ui::{Tab, TransferMsg};
use crate::view::kit;
use crate::view::menus_search::{IMPORT_TIP, import_scope};
use crate::view::popup::{close, dismissed, popup};

/// The menu that is open, under its button. `narrow` is the narrow header, whose Import lives in the Profile menu.
pub fn draw(
    state: &State,
    ui: &mut egui::Ui,
    anchors: &[(Menu, Rect)],
    narrow: bool,
    out: &mut Vec<Msg>,
) {
    let Some(open) = state.chrome.menu else {
        return;
    };
    let Some((_, anchor)) = anchors.iter().find(|(m, _)| *m == open) else {
        return;
    };
    let width = match open {
        Menu::Notifications => 420.0,
        Menu::History => 340.0,
        Menu::Vaults => 340.0,
        Menu::Profile => 340.0,
    };
    let at = egui::pos2(
        (anchor.right() - width).max(anchor.left().min(8.0)),
        anchor.bottom() + 4.0,
    );
    let at = if matches!(open, Menu::Vaults | Menu::History) {
        anchor.left_bottom() + egui::vec2(0.0, 4.0)
    } else {
        at
    };
    let rect = popup(ui, "navbar-menu", at, width, |ui| match open {
        Menu::Vaults => super::vault_menu::draw(state, ui, out),
        Menu::Notifications => crate::view::notifications::draw(state, ui, out),
        Menu::Profile => profile(state, ui, out, narrow),
        Menu::History => history(state, ui, out),
    });
    if dismissed(ui, rect, *anchor) {
        close(out);
    }
}

/// The places visited, newest first; the current one is marked.
fn history(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let h = &state.history;
    ui.strong("History");
    if h.entries.is_empty() {
        ui.weak("Nothing yet: the places you visit are listed here.");
    }
    for (i, place) in h.entries.iter().enumerate().rev() {
        let text = crate::history::label(state, place);
        let width = ui.available_width();
        let row = egui::Button::selectable(i == h.at, text).min_size(egui::vec2(width, 26.0));
        if ui
            .add(row)
            .on_hover_text("Goes back to this place you visited.")
            .clicked()
        {
            let go = crate::history::HistoryMsg::Go(i);
            out.push(Msg::Chrome(ChromeMsg::History(go)));
            close(out);
        }
    }
}

fn profile(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>, narrow: bool) {
    let p = crate::theme::palette_of(ui);
    ui.horizontal(|ui| {
        ui.strong(format!("StudyFrog {}", env!("CARGO_PKG_VERSION")));
        super::kit::pill(ui, "Workspace", p.text_weak);
    });
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(2.0);
    if narrow {
        // The header has no room for the Import button at this width, so it is listed here (BUG-0021).
        let import = kit::quiet_button(ui, "Import...").on_hover_text(IMPORT_TIP);
        if import.clicked() {
            out.push(Msg::Library(LibraryMsg::Transfer(TransferMsg::Open(
                Tab::Import,
                import_scope(state),
            ))));
            close(out);
        }
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(2.0);
    }
    ui.weak("Appearance");
    super::settings_appearance::theme_choice(state, ui, out);
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(2.0);
    super::profiles::draw(state, ui, out);
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);
    let close_btn = egui::Button::new(egui::RichText::new("Close vault").color(p.danger).strong())
        .min_size(egui::vec2(ui.available_width(), 26.0));
    if ui
        .add(close_btn)
        .on_hover_text("Closes the vault.")
        .clicked()
    {
        out.push(Msg::CloseVault);
        close(out);
    }
}
