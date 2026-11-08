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
        Menu::History => 340.0,
        Menu::Vaults => 300.0,
        _ => 260.0,
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
        Menu::Vaults => vaults(state, ui, out),
        Menu::Notifications => crate::view::notifications::draw(state, ui, out),
        Menu::Profile => profile(state, ui, out),
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
        if ui.add(row).clicked() {
            let go = crate::history::HistoryMsg::Go(i);
            out.push(Msg::Chrome(ChromeMsg::History(go)));
            close(out);
        }
    }
}

fn segments(path: &str) -> Vec<&str> {
    path.split(['/', '\\']).filter(|s| !s.is_empty()).collect()
}

/// Short, human-readable label for a vault path; appends a disambiguating ancestor when
/// another recent vault has the same folder name.
pub fn vault_label(path: &str, recent: &[String]) -> String {
    let segs = segments(path);
    let Some(&name) = segs.last() else {
        return path.to_owned();
    };
    let clashes: Vec<Vec<&str>> = recent
        .iter()
        .filter(|p| p.as_str() != path)
        .map(|p| segments(p))
        .filter(|s| s.last() == Some(&name))
        .collect();
    if clashes.is_empty() {
        return name.to_owned();
    }
    for &anc in segs.iter().rev().skip(1) {
        if clashes
            .iter()
            .any(|c| !c.iter().rev().skip(1).any(|&x| x == anc))
        {
            return format!("{name} · {anc}");
        }
    }
    segs.iter()
        .rev()
        .nth(1)
        .map_or_else(|| name.to_owned(), |p| format!("{name} · {p}"))
}

fn vaults(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.strong("Vaults");
    if let Some(v) = &state.vault {
        let width = ui.available_width();
        let active = egui::Button::selectable(true, &v.name).min_size(egui::vec2(width, 28.0));
        ui.add(active).on_hover_text(&v.root);
        if let Some(&parent) = segments(&v.root).iter().rev().nth(1) {
            ui.weak(parent);
        }
        ui.add_space(4.0);
    }
    for path in state
        .app
        .recent
        .iter()
        .filter(|p| Some(p.as_str()) != state.vault.as_ref().map(|v| v.root.as_str()))
    {
        let label = vault_label(path, &state.app.recent);
        let width = ui.available_width();
        let row = egui::Button::selectable(false, label).min_size(egui::vec2(width, 28.0));
        if ui.add(row).on_hover_text(path.as_str()).clicked() {
            out.push(Msg::OpenRecent(path.clone()));
            close(out);
        }
    }
    ui.separator();
    let width = ui.available_width();
    let open_other =
        egui::Button::selectable(false, "Open another vault…").min_size(egui::vec2(width, 28.0));
    if ui.add(open_other).clicked() {
        out.push(Msg::CloseVault);
        close(out);
    }
}

fn profile(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.strong(format!("StudyFrog {}", env!("CARGO_PKG_VERSION")));
    ui.separator();
    ui.label("Theme");
    super::settings_appearance::theme_choice(state, ui, out);
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
            let stack = crate::item_look::stack_by_key(state, query);
            if let Some(s) = stack
                && ui.button(format!("stack · {}", s.title)).clicked()
            {
                out.push(Msg::OpenItem(s.uid));
                out.push(Msg::Chrome(ChromeMsg::SearchClear));
            }
            let none = state.chrome.results.is_empty() && state.chrome.pdf_results.is_empty();
            if none && stack.is_none() {
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
        let name = crate::pdf_names::label(&hit.path, &hit.title);
        let label = match hit.page {
            Some(p) => format!("PDF · {name} · page {p}\n{}", hit.snippet),
            None => format!("PDF · {name}"),
        };
        if ui.button(label).clicked() {
            let page = hit.page.map_or(0, |p| p - 1);
            out.push(Msg::Pdf(PdfMsg::Open(hit.path.clone(), page)));
            out.push(Msg::Chrome(ChromeMsg::SearchClear));
        }
    }
}
