//! The Import dialog: cards from a card type an extension registered, and from a file with a registered importer.

use eframe::egui::{self, RichText, vec2};

use crate::modal::ModalMsg;
use crate::msg::{DraftField, Msg};
use crate::state::{Selection, State};
use crate::theme::palette_of;
use crate::view::icons::{Glyph, icon_button};
use crate::view::kit;
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

fn card_types(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let chosen = &state.browser.card_type;
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Card type:").strong());
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
    ui.add_space(4.0);
    ui.weak(hint);
    ui.add_space(4.0);
    if let Some(text) = text_field(ui, "Source text", &state.browser.draft.front, "", 4) {
        out.push(Msg::DraftEdited(DraftField::Front, text));
    }
    ui.add_space(6.0);
    if ui
        .add(egui::Button::new("Add cards").min_size(vec2(110.0, 32.0)))
        .clicked()
    {
        out.push(Msg::AddCardClicked);
    }
}

fn from_file(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let kinds: Vec<String> = state
        .extensions
        .importers
        .iter()
        .map(|i| i.1.clone())
        .collect();
    ui.weak(format!("Reads: {}", kinds.join("; ")));
    ui.add_space(6.0);
    if let Some(text) = text_field(
        ui,
        "File to import",
        &state.browser.import_path,
        "/path/to/cards.csv or .tsv",
        1,
    ) {
        out.push(Msg::ImportPathEdited(text));
    }
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new("Browse…").min_size(vec2(100.0, 32.0)))
            .clicked()
        {
            out.push(Msg::BrowseImportClicked);
        }
        ui.add_space(6.0);
        if ui
            .add(egui::Button::new("Import cards").min_size(vec2(110.0, 32.0)))
            .clicked()
        {
            out.push(Msg::ImportClicked);
        }
    });
}

/// The dialog.
pub fn import(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("Import into {}", target(state)))
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if icon_button(ui, Glyph::Close, "Close", false).clicked() {
                out.push(Msg::Modal(ModalMsg::Close));
            }
        });
    });
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .id_salt("modal-import-scroll")
        .max_height((ui.ctx().content_rect().height() - 140.0).clamp(320.0, 680.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if state.extensions.card_types.is_empty() && state.extensions.importers.is_empty() {
                ui.label("Nothing to import with yet: importers and card types come from extensions. Cards, notes and questions are made with Create.");
                ui.add_space(8.0);
            }
            if !state.extensions.card_types.is_empty() {
                kit::card(ui, |ui| {
                    ui.label(RichText::new("From a card type").size(crate::typo::size(ui, crate::typo::Tier::Heading)).strong());
                    ui.add_space(6.0);
                    card_types(state, ui, out);
                });
                ui.add_space(8.0);
            }
            if !state.extensions.importers.is_empty() {
                kit::card(ui, |ui| {
                    ui.label(RichText::new("From a file").size(crate::typo::size(ui, crate::typo::Tier::Heading)).strong());
                    ui.add_space(6.0);
                    from_file(state, ui, out);
                });
                ui.add_space(8.0);
            }
            kit::card(ui, |ui| {
                ui.label(RichText::new("From Obsidian, Anki or Markdown folder").size(crate::typo::size(ui, crate::typo::Tier::Heading)).strong());
                ui.add_space(4.0);
                ui.label(RichText::new("Import complete hierarchies of notes, flashcards, questions and stacks.").color(palette_of(ui).text_weak));
                ui.add_space(8.0);
                let btn = egui::Button::new(
                    RichText::new("Import notes, stacks and questions…")
                        .strong()
                        .color(palette_of(ui).on_accent),
                )
                .fill(palette_of(ui).accent)
                .min_size(vec2(220.0, 34.0));

                if ui.add(btn).clicked() {
                    let scope = match &state.browser.selection {
                        Selection::Stack(folder) => Some(folder.clone()),
                        _ => None,
                    };
                    out.push(Msg::Library(crate::library_ui::LibraryMsg::Transfer(
                        crate::transfer_ui::TransferMsg::Open(crate::transfer_ui::Tab::Import, scope),
                    )));
                }
            });
        });
}
