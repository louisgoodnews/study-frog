//! What the window shows before a vault is open. Choosing the vault is not the first thing asked:
//! an in-app notification (see `toast`) invites the user to set it up, and this page has the same
//! door.

use eframe::egui;

use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::state::State;

/// A greeting, the way into the vault dialog, and recent vaults.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let size = egui::vec2(520.0, ui.available_height());
    ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::Min), |ui| {
    ui.add_space(16.0);
    ui.heading("Welcome to StudyFrog");
    ui.add(
        egui::Label::new("Write notes in Markdown, turn them into flashcards, and rehearse them with spaced repetition. Everything stays in a folder on this computer.")
            .wrap(),
    );
    ui.add_space(12.0);
    if state.opening {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Opening and indexing the vault…");
        });
    } else if ui.button("Open or create a vault…").clicked() {
        out.push(Msg::Modal(ModalMsg::OpenVault));
    }
    if !state.app.recent.is_empty() {
        ui.add_space(12.0);
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
    });
}
