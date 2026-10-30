//! The items of the selected stack (or the inbox), with a title filter and the new-card form.

use eframe::egui::{self, Grid};
use studyfrog_core::plan::{ItemKind, item_kind_name};
use studyfrog_core::time::UnixMs;
use studyfrog_md::rfc3339::format_rfc3339;

use crate::browse::filter_items;
use crate::msg::{DraftField, Msg};
use crate::state::{Selection, State};
use crate::view::widgets::text_field;

/// "new" for an item never reviewed, else the UTC date it is next due.
fn due_text(due: Option<UnixMs>) -> String {
    match due.and_then(format_rfc3339) {
        Some(text) => format!("due {}", &text[..10]),
        None => "new".to_owned(),
    }
}

fn kind_text(kind: ItemKind) -> &'static str {
    item_kind_name(kind)
}

/// Heading, filter, item list, and the card form for a stack.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let b = &state.browser;
    let heading = match &b.selection {
        Selection::Stack(folder) => state
            .vault
            .as_ref()
            .and_then(|v| v.overview.stacks.iter().find(|s| s.folder == *folder))
            .map_or_else(|| folder.clone(), |s| s.title.clone()),
        _ => "Inbox".to_owned(),
    };
    ui.heading(heading);
    if let Some(text) = text_field(ui, "Search titles", &b.search, "type to filter", 1) {
        out.push(Msg::SearchEdited(text));
    }
    let shown = filter_items(&b.items, &b.search);
    ui.label(format!("{} of {} items", shown.len(), b.items.len()));
    egui::ScrollArea::vertical()
        .max_height(ui.available_height() * 0.5)
        .show(ui, |ui| {
            Grid::new("items")
                .striped(true)
                .num_columns(4)
                .show(ui, |ui| {
                    for item in &shown {
                        if ui.link(&item.title).clicked() {
                            out.push(Msg::OpenItem(item.uid));
                        }
                        ui.label(kind_text(item.kind));
                        ui.label(due_text(item.due));
                        ui.weak(&item.path);
                        ui.end_row();
                    }
                });
        });
    if matches!(b.selection, Selection::Stack(_)) {
        card_form(state, ui, out);
    }
}

fn card_form(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let (b, d) = (&state.browser, &state.browser.draft);
    ui.separator();
    ui.heading("New card");
    card_types(state, ui, out);
    let hint = b
        .card_type
        .as_deref()
        .and_then(|id| state.extensions.card_types.iter().find(|t| t.0 == id))
        .map(|t| t.2.clone());
    let fields: Vec<(DraftField, &str, &String, usize)> = match &hint {
        Some(_) => vec![(DraftField::Front, "Source text", &d.front, 4)],
        None => vec![
            (DraftField::Title, "Card title (optional)", &d.title, 1),
            (DraftField::Front, "Card front", &d.front, 3),
            (DraftField::Back, "Card back", &d.back, 3),
        ],
    };
    if let Some(h) = &hint {
        ui.weak(h);
    }
    for (field, label, current, rows) in fields {
        if let Some(text) = text_field(ui, label, current, "", rows) {
            out.push(Msg::DraftEdited(field, text));
        }
    }
    if ui
        .button(if hint.is_some() {
            "Add cards"
        } else {
            "Add card"
        })
        .clicked()
    {
        out.push(Msg::AddCardClicked);
    }
    import_panel(state, ui, out);
}

/// "Basic" and every registered card type, as a row of choices (only when extensions added some).
fn card_types(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.extensions.card_types.is_empty() {
        return;
    }
    let chosen = &state.browser.card_type;
    ui.horizontal_wrapped(|ui| {
        ui.label("Card type:");
        if ui
            .selectable_label(chosen.is_none(), "Basic (front and back)")
            .clicked()
        {
            out.push(Msg::CardTypeChosen(None));
        }
        for (id, name, _) in &state.extensions.card_types {
            if ui
                .selectable_label(chosen.as_deref() == Some(id), name)
                .clicked()
            {
                out.push(Msg::CardTypeChosen(Some(id.clone())));
            }
        }
    });
}

/// Import cards from a file with one of the registered importers.
fn import_panel(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.extensions.importers.is_empty() {
        return;
    }
    ui.separator();
    ui.heading("Import from a file");
    let kinds: Vec<String> = state
        .extensions
        .importers
        .iter()
        .map(|i| i.1.clone())
        .collect();
    ui.weak(format!("Reads: {}", kinds.join("; ")));
    if let Some(text) = text_field(
        ui,
        "File to import",
        &state.browser.import_path,
        "/path/to/file.tsv",
        1,
    ) {
        out.push(Msg::ImportPathEdited(text));
    }
    ui.horizontal(|ui| {
        if ui.button("Browse…").clicked() {
            out.push(Msg::BrowseImportClicked);
        }
        if ui.button("Import cards").clicked() {
            out.push(Msg::ImportClicked);
        }
    });
}
