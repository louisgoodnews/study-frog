//! Extension `CardType` expansion section inside the Universal Import Hub.

use eframe::egui::{self, RichText, vec2};

use crate::import_hub::{HubMsg, ImportDestination};
use crate::msg::{DraftField, Msg};
use crate::state::State;
use crate::theme::palette_of;
use crate::transfer_ui::TransferMsg;
use crate::view::kit;
use crate::view::widgets::text_field;

/// Draw the extension card-type section when extensions register at least one card type.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.extensions.card_types.is_empty() {
        return;
    }
    ui.add_space(6.0);
    kit::card(ui, |ui| {
        ui.label(RichText::new("From a card type").strong());
        ui.add_space(4.0);
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
        let Some(hint) = chosen
            .as_deref()
            .and_then(|id| state.extensions.card_types.iter().find(|t| t.0 == id))
            .map(|t| t.2.clone())
        else {
            return;
        };
        ui.add_space(4.0);
        ui.weak(hint);
        ui.add_space(4.0);
        if let Some(text) = text_field(ui, "Source text", &state.browser.draft.front, "", 4) {
            out.push(Msg::DraftEdited(DraftField::Front, text));
        }
        ui.add_space(6.0);
        let t = &state.transfer;
        let can_add = !t.busy
            && !state.browser.draft.front.trim().is_empty()
            && match t.hub.dest_mode {
                ImportDestination::ExistingStack => t.scope.is_some(),
                ImportDestination::NewStack => !t.hub.new_stack_name.trim().is_empty(),
                ImportDestination::UnstackedNotes => false,
            };
        if t.hub.dest_mode == ImportDestination::UnstackedNotes {
            ui.label(
                RichText::new("Select an existing or new stack above to add cards.")
                    .color(palette_of(ui).warning),
            );
        }
        if ui
            .add_enabled(
                can_add,
                egui::Button::new("Add cards").min_size(vec2(110.0, 32.0)),
            )
            .clicked()
        {
            out.push(Msg::Library(crate::library_ui::LibraryMsg::Transfer(
                TransferMsg::Hub(Box::new(HubMsg::CreateCardType)),
            )));
        }
    });
}
