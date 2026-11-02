//! The dialogs: create an item (see `modal_create`), import, and choose the vault.

use eframe::egui::{self, Align, Layout};

use crate::modal::{Modal, ModalMsg};
use crate::msg::Msg;
use crate::state::State;
use crate::view::icons::{Glyph, icon_button};
use crate::view::widgets::text_field;

pub(super) const WIDTH: f32 = 560.0;

fn send(out: &mut Vec<Msg>, m: ModalMsg) {
    out.push(Msg::Modal(m));
}

/// The open dialog, over a dimmed window; Escape or a click outside closes it.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(modal) = &state.modal else { return };
    let response = egui::Modal::new(egui::Id::unique("dialog")).show(ui.ctx(), |ui| {
        ui.set_width(match modal {
            Modal::Create(f) => super::modal_size::width(ui.ctx(), f.kind),
            _ => WIDTH,
        });
        match modal {
            Modal::Create(form) => super::modal_create::create(state, form, ui, out),
            Modal::Vault => vault(state, ui, out),
            Modal::Import => super::modal_import::import(state, ui, out),
        }
    });
    if response.should_close() {
        send(out, ModalMsg::Close);
    }
}

fn header(ui: &mut egui::Ui, out: &mut Vec<Msg>, add: impl FnOnce(&mut egui::Ui, &mut Vec<Msg>)) {
    ui.horizontal(|ui| {
        add(ui, out);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if icon_button(ui, Glyph::Close, "Close", false).clicked() {
                send(out, ModalMsg::Close);
            }
        });
    });
}

fn vault(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    header(ui, out, |ui, _| {
        ui.strong("Open or create a vault");
    });
    ui.add_space(6.0);
    ui.label("A vault is a folder of Markdown files that StudyFrog reads and writes. Choose an existing folder, or an empty one to start a new vault.");
    if let Some(t) = text_field(
        ui,
        "Vault folder",
        &state.path_input,
        "/path/to/your/vault",
        1,
    ) {
        out.push(Msg::PathEdited(t));
    }
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!state.opening, egui::Button::new("Choose folder…"))
            .clicked()
        {
            out.push(Msg::BrowseVaultClicked);
        }
        if ui
            .add_enabled(!state.opening, egui::Button::new("Open vault"))
            .clicked()
        {
            out.push(Msg::OpenVaultClicked);
        }
        if state.opening {
            ui.spinner();
        }
    });
    if !state.app.recent.is_empty() {
        ui.separator();
        ui.strong("Recent vaults");
        for path in &state.app.recent {
            if ui
                .add_enabled(!state.opening, egui::Button::new(path.as_str()))
                .clicked()
            {
                out.push(Msg::OpenRecent(path.clone()));
            }
        }
    }
}
