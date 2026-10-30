//! What the top-bar menus contain, and the search results under the search field.

use eframe::egui::{self, Rect, RichText};
use studyfrog_core::plan::item_kind_name;

use crate::chrome::{ChromeMsg, Creatable, Menu};
use crate::msg::Msg;
use crate::state::{NoticeKind, State};
use crate::view::popup::{close, dismissed, popup};

const SHORTCUTS: [(&str, &str); 8] = [
    ("Space", "show the answer"),
    ("1 2 3 4", "rate Again, Hard, Good, Easy"),
    ("E  M  D", "mark the item easy, medium or difficult"),
    ("S", "skip the item"),
    ("P", "pause or resume"),
    ("Esc", "stop the run, close a menu, clear the search"),
    ("Enter", "open the first search result"),
    ("Tab", "move between controls"),
];

/// The menu that is open, under its button.
pub fn draw(state: &State, ui: &mut egui::Ui, anchors: &[(Menu, Rect)], out: &mut Vec<Msg>) {
    let Some(open) = state.chrome.menu else {
        return;
    };
    let Some((_, anchor)) = anchors.iter().find(|(m, _)| *m == open) else {
        return;
    };
    let width = if open == Menu::Help { 340.0 } else { 260.0 };
    let at = egui::pos2(
        (anchor.right() - width).max(anchor.left().min(8.0)),
        anchor.bottom() + 4.0,
    );
    let at = if open == Menu::Vaults || open == Menu::Create {
        anchor.left_bottom() + egui::vec2(0.0, 4.0)
    } else {
        at
    };
    let rect = popup(ui, "navbar-menu", at, width, |ui| match open {
        Menu::Vaults => vaults(state, ui, out),
        Menu::Create => create(ui, out),
        Menu::Notifications => notifications(state, ui),
        Menu::Help => help(ui),
        Menu::Profile => profile(state, ui, out),
    });
    if dismissed(ui, rect, *anchor) {
        close(out);
    }
}

fn vaults(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.strong("Vaults");
    if let Some(v) = &state.vault {
        ui.label(format!("Open: {}", v.name));
    }
    for path in state
        .app
        .recent
        .iter()
        .filter(|p| Some(p.as_str()) != state.vault.as_ref().map(|v| v.root.as_str()))
    {
        if ui.button(path.as_str()).clicked() {
            out.push(Msg::OpenRecent(path.clone()));
            close(out);
        }
    }
    ui.separator();
    if ui.button("Open another vault…").clicked() {
        out.push(Msg::CloseVault);
        close(out);
    }
}

fn create(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    for (what, label) in [
        (Creatable::Stack, "New stack"),
        (Creatable::Card, "New card"),
        (Creatable::Session, "New study session"),
        (Creatable::BoardCard, "New board card"),
        (Creatable::Import, "Import cards from a file"),
    ] {
        if ui.button(label).clicked() {
            out.push(Msg::Chrome(ChromeMsg::Create(what)));
        }
    }
}

fn notifications(state: &State, ui: &mut egui::Ui) {
    ui.strong("Notifications");
    if state.chrome.notifications.is_empty() {
        ui.label("Nothing yet.");
    }
    for n in state.chrome.notifications.iter().rev().take(10) {
        let color = match n.kind {
            NoticeKind::Info => ui.visuals().text_color(),
            NoticeKind::Error => ui.visuals().error_fg_color,
        };
        ui.colored_label(color, &n.text);
    }
}

fn help(ui: &mut egui::Ui) {
    ui.strong("Keyboard");
    egui::Grid::new("shortcuts").num_columns(2).show(ui, |ui| {
        for (keys, what) in SHORTCUTS {
            ui.label(RichText::new(keys).monospace());
            ui.label(what);
            ui.end_row();
        }
    });
    ui.separator();
    ui.weak("Typed answers are compared by similarity: a high match is not a check of the facts.");
}

fn profile(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.strong(format!("StudyFrog {}", env!("CARGO_PKG_VERSION")));
    if let Some(v) = &state.vault {
        ui.weak(&v.root);
    }
    ui.separator();
    if ui.button("Close vault").clicked() {
        out.push(Msg::CloseVault);
        close(out);
    }
}

/// The matches for the search text, in a list under the field.
pub fn search_results(state: &State, ui: &mut egui::Ui, field: Rect, out: &mut Vec<Msg>) {
    let query = state.chrome.search.trim();
    if query.is_empty() {
        return;
    }
    let rect = popup(
        ui,
        "navbar-search",
        field.left_bottom() + egui::vec2(0.0, 4.0),
        field.width(),
        |ui| {
            if state.chrome.results.is_empty() {
                ui.weak(format!("No items match \"{query}\"."));
            }
            for item in &state.chrome.results {
                if ui
                    .button(format!("{} · {}", item_kind_name(item.kind), item.title))
                    .clicked()
                {
                    out.push(Msg::OpenItem(item.uid));
                    out.push(Msg::Chrome(ChromeMsg::SearchClear));
                }
            }
        },
    );
    if dismissed(ui, rect, field) {
        out.push(Msg::Chrome(ChromeMsg::SearchClear));
    }
}
