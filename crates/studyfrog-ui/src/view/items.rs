//! The items of the selected stack (or the inbox), with a title filter and the new-card form.

use eframe::egui::{self, Grid};
use studyfrog_core::plan::{ItemKind, item_kind_name};

use crate::browse::filter_items;
use crate::msg::{DraftField, Msg};
use crate::state::{Selection, State};
use crate::view::widgets::text_field;

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
                .num_columns(3)
                .show(ui, |ui| {
                    for item in &shown {
                        ui.label(&item.title);
                        ui.label(kind_text(item.kind));
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
    let d = &state.browser.draft;
    ui.separator();
    ui.heading("New card");
    let fields = [
        (DraftField::Title, "Card title (optional)", &d.title, 1),
        (DraftField::Front, "Card front", &d.front, 3),
        (DraftField::Back, "Card back", &d.back, 3),
    ];
    for (field, label, current, rows) in fields {
        if let Some(text) = text_field(ui, label, current, "", rows) {
            out.push(Msg::DraftEdited(field, text));
        }
    }
    if ui.button("Add card").clicked() {
        out.push(Msg::AddCardClicked);
    }
}
