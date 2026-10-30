//! Views: plain functions from `&State` to the messages the user's input produced.

mod achievements;
mod bars;
mod board;
mod calendar;
mod editor;
mod graph;
mod icons;
mod items;
mod library;
mod menus;
mod modal;
mod navbar;
mod popup;
mod preview;
mod reports;
mod run;
mod session_form;
mod settings;
mod setup;
mod sidebar;
mod summary;
mod toast;
mod welcome;
pub(crate) mod widgets;

use eframe::egui;

use crate::msg::Msg;
use crate::state::{Screen, State};

/// Draw the whole window for `state`.
pub fn draw(state: &State, ui: &mut egui::Ui) -> Vec<Msg> {
    let mut out = Vec::new();
    navbar::draw(state, ui, &mut out);
    sidebar::draw(state, ui, &mut out);
    bars::status(state, ui, &mut out);
    egui::CentralPanel::default().show(ui, |ui| match state.screen {
        Screen::Welcome => welcome::draw(state, ui, &mut out),
        Screen::Library => library::draw(state, ui, &mut out),
        Screen::Rehearse if state.rehearse.run.is_some() => run::draw(state, ui, &mut out),
        Screen::Rehearse => setup::draw(state, ui, &mut out),
        Screen::Settings => settings::draw(state, ui, &mut out),
        Screen::Editor => editor::draw(state, ui, &mut out),
        Screen::Graph => graph::draw(state, ui, &mut out),
        Screen::Calendar => calendar::draw(state, ui, &mut out),
        Screen::Board => board::draw(state, ui, &mut out),
        Screen::Achievements => achievements::draw(state, ui),
        Screen::Reports => reports::draw(state, ui),
    });
    toast::draw(state, ui, &mut out);
    modal::draw(state, ui, &mut out);
    out
}
