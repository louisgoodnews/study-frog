//! Views: plain functions from `&State` to the messages the user's input produced.

mod bars;
mod board;
mod calendar;
mod editor;
mod graph;
mod items;
mod library;
mod preview;
mod run;
mod session_form;
mod settings;
mod setup;
mod summary;
mod welcome;
pub(crate) mod widgets;

use eframe::egui;

use crate::msg::Msg;
use crate::state::{Screen, State};

/// Draw the whole window for `state`.
pub fn draw(state: &State, ui: &mut egui::Ui) -> Vec<Msg> {
    let mut out = Vec::new();
    if state.vault.is_some() {
        bars::nav(state, ui, &mut out);
    }
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
    });
    out
}
