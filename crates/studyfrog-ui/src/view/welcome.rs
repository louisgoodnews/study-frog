//! The vault chooser.

use eframe::egui::{self, Key, TextEdit};

use crate::msg::Msg;
use crate::state::State;

/// Path field, open button, recent vaults.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.heading("StudyFrog");
    ui.label("Open a folder of Markdown notes as your vault.");
    ui.add_space(8.0);
    let label = ui.label("Vault folder");
    let mut text = state.path_input.clone();
    let field = ui
        .add(
            TextEdit::singleline(&mut text)
                .hint_text("/path/to/your/vault")
                .desired_width(f32::INFINITY),
        )
        .labelled_by(label.id);
    if text != state.path_input {
        out.push(Msg::PathEdited(text));
    }
    let enter = field.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
    ui.horizontal(|ui| {
        let open = ui.add_enabled(!state.opening, egui::Button::new("Open vault"));
        if enter || open.clicked() {
            out.push(Msg::OpenVaultClicked);
        }
        if ui
            .add_enabled(!state.opening, egui::Button::new("Choose folder…"))
            .clicked()
        {
            out.push(Msg::BrowseVaultClicked);
        }
    });
    if state.opening {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Opening and indexing the vault…");
        });
    }
    recent(state, ui, out);
}

fn recent(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.app.recent.is_empty() {
        return;
    }
    ui.add_space(12.0);
    ui.heading("Recent vaults");
    for path in &state.app.recent {
        if ui
            .add_enabled(!state.opening, egui::Button::new(path.as_str()))
            .clicked()
        {
            out.push(Msg::OpenRecent(path.clone()));
        }
    }
}
