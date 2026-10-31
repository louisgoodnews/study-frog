//! Views: plain functions from `&State` to the messages the user's input produced.

mod achievements;
mod agent;
mod agent_exam;
mod agent_wiki;
mod backup;
mod bars;
mod board;
mod calendar;
mod calendar_grid;
mod date_picker;
mod editor;
mod explorer;
mod explorer_charts;
mod graph;
mod home;
mod home_cards;
mod icons;
pub(crate) mod image;
mod item_details;
mod items;
pub(crate) mod kit;
mod library;
mod library_pdfs;
mod md_editor;
mod menus;
mod modal;
mod modal_fields;
mod navbar;
mod pdf;
mod pdf_arrange;
mod pdf_extract;
mod pdf_notes;
mod pdf_page;
mod pdf_side;
mod pdf_texture;
mod popup;
mod preview;
mod profiles;
mod reports;
mod run;
mod run_card;
mod session_form;
mod settings;
mod setup;
mod setup_how;
mod setup_review;
mod setup_what;
mod sidebar;
mod subjects;
mod summary;
mod summary_items;
mod toast;
mod welcome;
pub(crate) mod widgets;

use eframe::egui;

use crate::msg::Msg;
use crate::state::{Screen, State};

/// Draw the whole window for `state`.
pub fn draw(state: &State, ui: &mut egui::Ui) -> Vec<Msg> {
    let mut out = Vec::new();
    image::set_vault_root(ui.ctx(), state.vault.as_ref().map(|v| v.root.as_str()));
    navbar::draw(state, ui, &mut out);
    sidebar::draw(state, ui, &mut out);
    bars::status(state, ui, &mut out);
    egui::CentralPanel::default().show(ui, |ui| match state.screen {
        Screen::Welcome => welcome::draw(state, ui, &mut out),
        Screen::Home => home::draw(state, ui, &mut out),
        Screen::Library => library::draw(state, ui, &mut out),
        Screen::Rehearse if state.rehearse.run.is_some() => run::draw(state, ui, &mut out),
        Screen::Rehearse => setup::draw(state, ui, &mut out),
        Screen::Settings => settings::draw(state, ui, &mut out),
        Screen::Editor => editor::draw(state, ui, &mut out),
        Screen::Graph => graph::draw(state, ui, &mut out),
        Screen::Calendar => calendar::draw(state, ui, &mut out),
        Screen::Board => board::draw(state, ui, &mut out),
        Screen::Achievements => achievements::draw(state, ui),
        Screen::Agent => agent::draw(state, ui, &mut out),
        Screen::Reports => reports::draw(state, ui, &mut out),
        Screen::Pdf => pdf::draw(state, ui, &mut out),
    });
    toast::draw(state, ui, &mut out);
    modal::draw(state, ui, &mut out);
    out
}
