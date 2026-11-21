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

/// Where the items go: as the file says, the vault root, an existing stack, or a new stack.
fn destination_menu(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    let stacks: Vec<_> = state
        .vault
        .iter()
        .flat_map(|v| v.overview.stacks.iter())
        .collect();
    let current = match (&t.new_stack, t.scope.as_deref()) {
        (Some(_), _) => "New stack…".to_owned(),
        (None, None) => "As the file says".to_owned(),
        (None, Some("")) => "Vault root (no stack)".to_owned(),
        (None, Some(folder)) => stacks
            .iter()
            .find(|s| s.folder == folder)
            .map_or_else(|| folder.to_owned(), |s| s.title.clone()),
    };
    egui::ComboBox::from_label("Import into")
        .selected_text(current)
        .show_ui(ui, |ui| {
            let as_file = t.scope.is_none() && t.new_stack.is_none();
            if ui.selectable_label(as_file, "As the file says").clicked() {
                send(out, TransferMsg::Scope(None));
            }
            let root = t.scope.as_deref() == Some("") && t.new_stack.is_none();
            if ui.selectable_label(root, "Vault root (no stack)").clicked() {
                send(out, TransferMsg::Scope(Some(String::new())));
            }
            for s in &stacks {
                let picked = t.scope.as_deref() == Some(s.folder.as_str()) && t.new_stack.is_none();
                if ui.selectable_label(picked, &s.title).clicked() {
                    send(out, TransferMsg::Scope(Some(s.folder.clone())));
                }
            }
            if ui
                .selectable_label(t.new_stack.is_some(), "New stack…")
                .clicked()
            {
                send(out, TransferMsg::ChooseNewStack);
            }
        });
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
    if let Some(name) = &t.new_stack
        && let Some(text) = text_field(ui, "New stack name", name, "e.g. Spanish/verbs", 1)
    {
        send(out, TransferMsg::NewStackName(text));
    }
    if !t.path.trim().is_empty() {
        let is_dir = std::path::Path::new(t.path.trim()).is_dir();
        let described = studyfrog_service::xfer_adapters::describe_input(&t.path, is_dir);
        ui.weak(described);
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new("Choose file…").min_size(vec2(110.0, 28.0)))
            .on_hover_text("Opens a dialog to pick a file.")
            .clicked()
        {
            send(out, TransferMsg::Browse(true, false));
        }
        ui.add_space(4.0);
        if ui
            .add(egui::Button::new("Choose folder…").min_size(vec2(120.0, 28.0)))
            .on_hover_text("Opens a dialog to pick a folder.")
            .clicked()
        {
            send(out, TransferMsg::Browse(true, true));
        }
        ui.add_space(8.0);
        destination_menu(state, ui, out);
        ui.add_space(8.0);

        let can_preview = !t.busy && !t.path.trim().is_empty();
        let preview_btn = egui::Button::new(
            RichText::new("Preview")
                .strong()
                .size(crate::typo::size(ui, crate::typo::Tier::Body)),
        )
        .fill(palette_of(ui).surface_hover)
        .min_size(vec2(90.0, 28.0));

        if ui
            .add_enabled(can_preview, preview_btn)
            .on_hover_text("Shows what the import would do, without writing anything.")
            .clicked()
        {
            send(out, TransferMsg::Preview);
        }
    });

    super::modal_transfer_preview::preview(state, ui, out);
}
