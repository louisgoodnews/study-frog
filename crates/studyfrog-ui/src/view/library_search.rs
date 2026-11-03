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
    let r = egui::ComboBox::from_id_salt(label)
        .selected_text(format!("{label}: {shown}"))
        .width(132.0)
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
    r.response
        .widget_info(|| egui::WidgetInfo::labeled(egui::Role::ComboBox, true, label));
    picked
}

/// A single-line field of the given width with a name for screen readers; the new text when it changed.
fn field(ui: &mut egui::Ui, (name, hint): (&str, &str), text: &str, width: f32) -> Option<String> {
    let mut edited = text.to_owned();
    let r = ui.add(
        egui::TextEdit::singleline(&mut edited)
            .hint_text(hint)
            .desired_width(width),
    );
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, name));
    (edited != text).then_some(edited)
}

/// Room kept for the three drop-downs after the search field.
const COMBOS_WIDTH: f32 = 3.0 * 140.0;

fn standard(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let s = &state.library.search;
    let width = (ui.available_width() - COMBOS_WIDTH).max(160.0);
    if let Some(text) = field(
        ui,
        ("Search titles", "Search titles"),
        &state.browser.search,
        width,
    ) {
        out.push(Msg::SearchEdited(text));
    }
    let kinds = [ItemKind::Note, ItemKind::Card, ItemKind::Question].map(|k| (k, kind_label(k)));
    if let Some(k) = choice(ui, "Type", s.kind, &kinds) {
        send(out, SearchMsg::Kind(k));
    }
    let statuses: Vec<(Status, &str)> = STATUSES.iter().map(|v| (*v, status_label(*v))).collect();
    if let Some(v) = choice(ui, "Status", s.status, &statuses) {
        send(out, SearchMsg::Status(v));
    }
    let priorities: Vec<(Priority, &str)> =
        PRIORITIES.iter().map(|p| (*p, priority_name(*p))).collect();
    if let Some(p) = choice(ui, "Priority", s.priority, &priorities) {
        send(out, SearchMsg::Priority(p));
    }
}

/// The query box on the row, and under it its error or the suggestions.
fn query_field(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let width = ui.available_width();
    let hint = "status = new AND due <= 7d ORDER BY due";
    if let Some(text) = field(ui, ("Query text", hint), &state.library.search.query, width) {
        send(out, SearchMsg::Query(text));
    }
}

fn query_help(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let lib = &state.library;
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

/// The bar: the mode switch, then the fields or the query box on the same row.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mode = state.library.search.mode;
    ui.horizontal(|ui| {
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
        ui.add_space(6.0);
        match mode {
            SearchMode::Standard => standard(state, ui, out),
            SearchMode::Query => query_field(state, ui, out),
        }
    });
    if mode == SearchMode::Query {
        query_help(state, ui, out);
    }
}
