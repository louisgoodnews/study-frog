//! The navigation bar and the status bar.

use eframe::egui::{self, Panel};

use crate::msg::Msg;
use crate::state::{NoticeKind, Screen, State};

const SCREENS: [(Screen, &str); 6] = [
    (Screen::Library, "Library"),
    (Screen::Calendar, "Calendar"),
    (Screen::Board, "Board"),
    (Screen::Graph, "Graph"),
    (Screen::Rehearse, "Rehearse"),
    (Screen::Settings, "Settings"),
];

/// Screen buttons on the left, "Close vault" on the right.
pub fn nav(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    Panel::top("nav").show(ui, |ui| {
        ui.horizontal(|ui| {
            for (screen, label) in SCREENS {
                if ui.selectable_label(state.screen == screen, label).clicked() {
                    out.push(Msg::Navigate(screen));
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close vault").clicked() {
                    out.push(Msg::CloseVault);
                }
            });
        });
    });
}

/// The latest notice with a dismiss button, or the vault path when there is none.
pub fn status(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    Panel::bottom("status").show(ui, |ui| {
        ui.horizontal(|ui| match &state.notice {
            Some(n) => {
                let color = match n.kind {
                    NoticeKind::Info => ui.visuals().text_color(),
                    NoticeKind::Error => ui.visuals().error_fg_color,
                };
                ui.colored_label(color, &n.text);
                if ui.button("Dismiss").clicked() {
                    out.push(Msg::DismissNotice);
                }
            }
            None => {
                let text = state
                    .vault
                    .as_ref()
                    .map_or("No vault open", |v| v.root.as_str());
                ui.weak(text);
            }
        });
    });
}
