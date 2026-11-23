//! Central-panel screen dispatch extracted from `view/mod.rs` to keep module size bounded.

use eframe::egui;

use crate::msg::Msg;
use crate::state::{Screen, State};

/// Draw the active screen inside the central panel.
pub fn draw_screen(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    match state.screen {
        Screen::Welcome => super::welcome::draw(state, ui, out),
        Screen::Home => super::home::draw(state, ui, out),
        Screen::Folders => super::folders::draw(state, ui, out),
        Screen::Library => super::library::draw(state, ui, out),
        Screen::Rehearse if crate::setup::memory::run_on_screen(state) => {
            if state.split_pdf.active {
                super::pdf_split::draw_split_rehearse(state, ui, out);
            } else {
                super::run::draw(state, ui, out);
            }
        }
        Screen::Rehearse => super::setup::draw(state, ui, out),
        Screen::Settings => super::settings::draw(state, ui, out),
        Screen::Editor => {
            if state.split_pdf.active {
                super::pdf_split::draw_split_editor(state, ui, out);
            } else {
                super::editor::draw(state, ui, out);
            }
        }
        Screen::Graph => super::graph::draw(state, ui, out),
        Screen::Calendar => super::calendar::draw(state, ui, out),
        Screen::Homework => super::homework::draw(state, ui, out),
        Screen::Board => super::board::draw(state, ui, out),
        Screen::Achievements => super::achievements::draw(state, ui, out),
        Screen::Agent => super::agent::draw(state, ui, out),
        Screen::Prompts => super::prompts_screen::draw(state, ui, out),
        Screen::Reports => super::reports::draw(state, ui, out),
        Screen::Diary => super::diary::draw(state, ui, out),
        Screen::Pdf => super::pdf::draw(state, ui, out),
        Screen::Help => super::help::draw(state, ui, out),
        Screen::Console => super::console::draw(state, ui, out),
        Screen::Canvas => super::canvas::draw(state, ui, out),
        Screen::Match => super::match_grid::draw(state, ui, out),
    }
}
