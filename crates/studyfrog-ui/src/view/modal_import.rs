//! The Import dialog: cards from a card type an extension registered, and from a file with a registered importer.

use eframe::egui;

use crate::modal::ModalMsg;
use crate::msg::{DraftField, Msg};
use crate::state::{Selection, State};
use crate::view::icons::{Glyph, icon_button};
use crate::view::widgets::text_field;

fn target(state: &State) -> String {
    let Selection::Stack(folder) = &state.browser.selection else {
        return String::new();
    };
    state
        .vault
        .as_ref()
        .and_then(|v| v.overview.stacks.iter().find(|s| s.folder == *folder))
        .map_or_else(|| folder.clone(), |s| s.title.clone())
}

/// The card types extensions registered, as a row of choices; choosing the chosen one again goes back to none.
fn card_types(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let chosen = &state.browser.card_type;
    ui.horizontal_wrapped(|ui| {
        ui.label("Card type:");
        for (id, name, _) in &state.extensions.card_types {
            let on = chosen.as_deref() == Some(id);
            if ui.selectable_label(on, name).clicked() {
                out.push(Msg::CardTypeChosen((!on).then(|| id.clone())));
            }
        }
    });
    let hint = chosen
        .as_deref()
        .and_then(|id| state.extensions.card_types.iter().find(|t| t.0 == id))
        .map(|t| t.2.clone());
    let Some(hint) = hint else { return };
    ui.weak(hint);
    if let Some(text) = text_field(ui, "Source text", &state.browser.draft.front, "", 4) {
        out.push(Msg::DraftEdited(DraftField::Front, text));
    }
    if ui.button("Add cards").clicked() {
        out.push(Msg::AddCardClicked);
    }
}

/// Import cards from a file with one of the registered importers.
fn from_file(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
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

/// The dialog.
pub fn import(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.strong(format!("Import into {}", target(state)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if icon_button(ui, Glyph::Close, "Close", false).clicked() {
                out.push(Msg::Modal(ModalMsg::Close));
            }
        });
    });
    if state.extensions.card_types.is_empty() && state.extensions.importers.is_empty() {
        ui.label("Nothing to import with yet: importers and card types come from extensions. Cards, notes and questions are made with Create.");
        return;
    }
    if !state.extensions.card_types.is_empty() {
        ui.add_space(6.0);
        ui.strong("From a card type");
        card_types(state, ui, out);
    }
    if !state.extensions.importers.is_empty() {
        ui.separator();
        ui.strong("From a file");
        from_file(state, ui, out);
    }
}
