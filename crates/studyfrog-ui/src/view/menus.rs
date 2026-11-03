//! What the top-bar menus contain, and the search results under the search field.

use eframe::egui::{self, Rect};
use studyfrog_core::plan::item_kind_name;

use crate::chrome::{ChromeMsg, Menu};
use crate::msg::Msg;
use crate::pdf_types::PdfMsg;
use crate::state::State;
use crate::view::popup::{close, dismissed, popup};

/// The menu that is open, under its button.
pub fn draw(state: &State, ui: &mut egui::Ui, anchors: &[(Menu, Rect)], out: &mut Vec<Msg>) {
    let Some(open) = state.chrome.menu else {
        return;
    };
    let Some((_, anchor)) = anchors.iter().find(|(m, _)| *m == open) else {
        return;
    };
    let width = match open {
        Menu::Notifications => 420.0,
        _ => 260.0,
    };
    let at = egui::pos2(
        (anchor.right() - width).max(anchor.left().min(8.0)),
        anchor.bottom() + 4.0,
    );
    let at = if open == Menu::Vaults {
        anchor.left_bottom() + egui::vec2(0.0, 4.0)
    } else {
        at
    };
    let rect = popup(ui, "navbar-menu", at, width, |ui| match open {
        Menu::Vaults => vaults(state, ui, out),
        Menu::Notifications => crate::view::notifications::draw(state, ui, out),
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

fn profile(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.strong(format!("StudyFrog {}", env!("CARGO_PKG_VERSION")));
    ui.separator();
    super::profiles::draw(state, ui, out);
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
            if state.chrome.results.is_empty() && state.chrome.pdf_results.is_empty() {
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
            pdf_hits(state, ui, out);
        },
    );
    if dismissed(ui, rect, field) {
        out.push(Msg::Chrome(ChromeMsg::SearchClear));
    }
}

/// PDFs and pages that match the search, under the items.
fn pdf_hits(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    for hit in &state.chrome.pdf_results {
        let label = match hit.page {
            Some(p) => format!("PDF · {} · page {p}\n{}", hit.title, hit.snippet),
            None => format!("PDF · {}", hit.title),
        };
        if ui.button(label).clicked() {
            let page = hit.page.map_or(0, |p| p - 1);
            out.push(Msg::Pdf(PdfMsg::Open(hit.path.clone(), page)));
            out.push(Msg::Chrome(ChromeMsg::SearchClear));
        }
    }
}
