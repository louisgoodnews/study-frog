//! 3-option Target Destination Picker for the Universal Import Hub.

use eframe::egui::{self, ComboBox, RichText};

use crate::import_hub::{HubMsg, ImportDestination};
use crate::msg::Msg;
use crate::state::State;
use crate::transfer_ui::TransferMsg;
use crate::view::kit;
use crate::view::widgets::text_field;

fn send_transfer(out: &mut Vec<Msg>, m: TransferMsg) {
    out.push(Msg::Library(crate::library_ui::LibraryMsg::Transfer(m)));
}

fn send_hub(out: &mut Vec<Msg>, m: HubMsg) {
    send_transfer(out, TransferMsg::Hub(Box::new(m)));
}

fn existing_stack_combo(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let stacks: Vec<_> = state
        .vault
        .iter()
        .flat_map(|v| v.overview.stacks.iter())
        .collect();
    let current = state
        .transfer
        .scope
        .as_ref()
        .and_then(|f| stacks.iter().find(|s| s.folder == *f))
        .map_or_else(|| "As the file says".to_owned(), |s| s.title.clone());
    ComboBox::from_label("Target stack")
        .selected_text(current)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(state.transfer.scope.is_none(), "As the file says")
                .clicked()
            {
                send_transfer(out, TransferMsg::Scope(None));
            }
            for s in &stacks {
                if ui
                    .selectable_label(state.transfer.scope.as_deref() == Some(&s.folder), &s.title)
                    .clicked()
                {
                    send_transfer(out, TransferMsg::Scope(Some(s.folder.clone())));
                }
            }
        });
}

/// Draw the 3-option destination picker (`Existing stack`, `New stack`, `Unstacked vault notes`).
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    let mode = t.hub.dest_mode;
    kit::card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Destination:").strong());
            for (kind, label) in [
                (ImportDestination::ExistingStack, "Existing stack"),
                (ImportDestination::NewStack, "New stack"),
                (ImportDestination::UnstackedNotes, "Unstacked vault notes"),
            ] {
                if ui.radio(mode == kind, label).clicked() {
                    send_hub(out, HubMsg::SetDest(kind));
                }
            }
        });
        ui.add_space(4.0);
        match mode {
            ImportDestination::ExistingStack => existing_stack_combo(state, ui, out),
            ImportDestination::NewStack => {
                if let Some(name) = text_field(
                    ui,
                    "New stack name",
                    &t.hub.new_stack_name,
                    "e.g. Biology 101",
                    1,
                ) {
                    send_hub(out, HubMsg::EditNewStack(name));
                }
            }
            ImportDestination::UnstackedNotes => {
                ui.weak("Saved at the vault root (Inbox) without a stack.");
            }
        }
    });
}
