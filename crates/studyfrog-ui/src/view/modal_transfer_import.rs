//! The Import tab of the transfer modal: discover, pick file/folder, pick scope, preview and import.

use eframe::egui::{self, RichText, vec2};

use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::transfer_ui::TransferMsg;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: TransferMsg) {
    out.push(Msg::Library(crate::library_ui::LibraryMsg::Transfer(m)));
}

/// The import tab content.
pub fn import_tab(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;

    super::modal_found::found(state, ui, out);
    super::modal_formats::formats(ui, out);
    ui.add_space(2.0);

    if let Some(text) = text_field(ui, "File or folder to import", &t.path, "/path/to/file", 1) {
        send(out, TransferMsg::Path(text));
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new("Choose file…").min_size(vec2(110.0, 28.0)))
            .clicked()
        {
            send(out, TransferMsg::Browse(true, false));
        }
        ui.add_space(4.0);
        if ui
            .add(egui::Button::new("Choose folder…").min_size(vec2(120.0, 28.0)))
            .clicked()
        {
            send(out, TransferMsg::Browse(true, true));
        }
        ui.add_space(8.0);
        super::modal_transfer::scope_menu(state, ui, out, "Import");
        ui.add_space(8.0);
        let can_preview = !t.busy && !t.path.trim().is_empty();
        let preview_btn = egui::Button::new(
            RichText::new("Preview")
                .strong()
                .size(crate::typo::size(ui, crate::typo::Tier::Body)),
        )
        .fill(palette_of(ui).surface_hover)
        .min_size(vec2(90.0, 28.0));

        if ui.add_enabled(can_preview, preview_btn).clicked() {
            send(out, TransferMsg::Preview);
        }
    });

    super::modal_transfer_preview::preview(state, ui, out);
}
