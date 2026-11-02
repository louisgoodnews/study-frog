//! The search bar above the library table: Standard fields or a query box with its errors and suggestions.

use eframe::egui::{self, RichText};
use studyfrog_core::plan::ItemKind;
use studyfrog_core::priority::{PRIORITIES, Priority, priority_name};
use studyfrog_core::query_complete::complete;
use studyfrog_core::status::{STATUSES, Status, status_label};

use crate::library_search::{SearchMode, SearchMsg};
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: SearchMsg) {
    out.push(Msg::Library(LibraryMsg::Search(m)));
}

fn kind_label(k: ItemKind) -> &'static str {
    match k {
        ItemKind::Note => "Note",
        ItemKind::Card => "Card",
        ItemKind::Question => "Question",
    }
}

/// A drop-down of "Any" and the options; the new choice when it changed.
fn choice<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    label: &str,
    current: Option<T>,
    options: &[(T, &str)],
) -> Option<Option<T>> {
    let shown = options
        .iter()
        .find(|(v, _)| Some(*v) == current)
        .map_or("Any", |(_, name)| *name);
    let mut picked = None;
    egui::ComboBox::from_label(label)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            if ui.selectable_label(current.is_none(), "Any").clicked() {
                picked = Some(None);
            }
            for (v, name) in options {
                if ui.selectable_label(current == Some(*v), *name).clicked() {
                    picked = Some(Some(*v));
                }
            }
        });
    picked
}

fn standard(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let s = &state.library.search;
    if let Some(text) = text_field(
        ui,
        "Search titles",
        &state.browser.search,
        "type to filter",
        1,
    ) {
        out.push(Msg::SearchEdited(text));
    }
    ui.horizontal_wrapped(|ui| {
        let kinds =
            [ItemKind::Note, ItemKind::Card, ItemKind::Question].map(|k| (k, kind_label(k)));
        if let Some(k) = choice(ui, "Type", s.kind, &kinds) {
            send(out, SearchMsg::Kind(k));
        }
        let statuses: Vec<(Status, &str)> =
            STATUSES.iter().map(|v| (*v, status_label(*v))).collect();
        if let Some(v) = choice(ui, "Status", s.status, &statuses) {
            send(out, SearchMsg::Status(v));
        }
        let priorities: Vec<(Priority, &str)> =
            PRIORITIES.iter().map(|p| (*p, priority_name(*p))).collect();
        if let Some(p) = choice(ui, "Priority", s.priority, &priorities) {
            send(out, SearchMsg::Priority(p));
        }
    });
}

fn query(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let lib = &state.library;
    if let Some(text) = text_field(
        ui,
        "Query text",
        &lib.search.query,
        "status = new AND due <= 7d ORDER BY due",
        1,
    ) {
        send(out, SearchMsg::Query(text));
    }
    if let Some(e) = &lib.query_error {
        let red = palette_of(ui).danger;
        ui.label(RichText::new(format!("Column {}: {}", e.column, e.message)).color(red));
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for (n, c) in complete(&lib.search.query).iter().enumerate() {
            if ui.small_button(format!("Insert {}", c.text)).clicked() {
                send(out, SearchMsg::Complete(n));
            }
        }
    });
}

/// The bar: the mode switch and the fields or the query box.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mode = state.library.search.mode;
    ui.horizontal(|ui| {
        ui.label("Search mode");
        if ui
            .selectable_label(mode == SearchMode::Standard, "Standard")
            .clicked()
            && mode != SearchMode::Standard
        {
            send(out, SearchMsg::Mode(false));
        }
        if ui
            .selectable_label(mode == SearchMode::Query, "Query")
            .clicked()
            && mode != SearchMode::Query
        {
            send(out, SearchMsg::Mode(true));
        }
    });
    match mode {
        SearchMode::Standard => standard(state, ui, out),
        SearchMode::Query => query(state, ui, out),
    }
}
