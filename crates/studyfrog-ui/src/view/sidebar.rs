//! The sidebar: grouped screen navigation with visual breathing room.

use eframe::egui::{self, Panel, RichText};

use crate::msg::Msg;
use crate::state::{Screen, State};
use crate::theme::palette_of;

struct Group {
    caption: Option<&'static str>,
    screens: &'static [(Screen, &'static str)],
}

const GROUPS: &[Group] = &[
    Group {
        caption: Some("Study"),
        screens: &[
            (Screen::Home, "Home"),
            (Screen::Folders, "Folders"),
            (Screen::Library, "Items"),
            (Screen::Rehearse, "Rehearse"),
        ],
    },
    Group {
        caption: Some("Plan"),
        screens: &[(Screen::Calendar, "Calendar"), (Screen::Board, "Board")],
    },
    Group {
        caption: Some("Insight"),
        screens: &[
            (Screen::Reports, "Reports"),
            (Screen::Diary, "Diary"),
            (Screen::Achievements, "Achievements"),
            (Screen::Graph, "Graph"),
        ],
    },
    Group {
        caption: None,
        screens: &[
            (Screen::Agent, "Assistant"),
            (Screen::Settings, "Settings"),
            (Screen::Help, "Help"),
        ],
    },
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
            let p = palette_of(ui);
            ui.add_space(4.0);
            for (i, group) in GROUPS.iter().enumerate() {
                if let Some(caption) = group.caption {
                    if i > 0 {
                        ui.add_space(8.0);
                    }
                    ui.label(
                        RichText::new(caption)
                            .size(11.0)
                            .strong()
                            .color(p.text_weak),
                    );
                    ui.add_space(2.0);
                } else {
                    ui.add_space(10.0);
                }
                for &(screen, label) in group.screens {
                    let width = ui.available_width();
                    let button = egui::Button::selectable(state.screen == screen, label)
                        .min_size(egui::vec2(width, 26.0));
                    if ui.add(button).clicked() {
                        out.push(Msg::Navigate(screen));
                    }
                }
            }
        });
}
