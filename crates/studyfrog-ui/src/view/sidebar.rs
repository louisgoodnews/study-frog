//! The sidebar: where each screen is.

use eframe::egui::{self, Panel};

use crate::msg::Msg;
use crate::state::{Screen, State};

const SCREENS: [(Screen, &str); 9] = [
    (Screen::Home, "Home"),
    (Screen::Library, "Library"),
    (Screen::Calendar, "Calendar"),
    (Screen::Board, "Board"),
    (Screen::Achievements, "Achievements"),
    (Screen::Reports, "Reports"),
    (Screen::Graph, "Graph"),
    (Screen::Rehearse, "Rehearse"),
    (Screen::Settings, "Settings"),
];

/// The screen list on the left, shown while a vault is open and the sidebar is not folded away.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.vault.is_none() || state.chrome.sidebar_hidden {
        return;
    }
    Panel::left("sidebar")
        .resizable(false)
        .exact_size(150.0)
        .show(ui, |ui| {
            ui.add_space(6.0);
            for (screen, label) in SCREENS {
                let width = ui.available_width();
                let button = egui::Button::selectable(state.screen == screen, label)
                    .min_size(egui::vec2(width, 28.0));
                if ui.add(button).clicked() {
                    out.push(Msg::Navigate(screen));
                }
            }
        });
}
