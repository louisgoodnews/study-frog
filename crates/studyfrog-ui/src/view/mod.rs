//! Views: plain functions from `&State` to the messages the user's input produced.

mod achievements;
mod agent;
mod agent_exam;
mod agent_wiki;
mod backup;
mod bars;
mod board;
pub(crate) mod calendar;
mod calendar_grid;
mod calendar_reviews;
mod date_picker;
mod editor;
mod explorer;
mod explorer_charts;
mod graph;
mod help;
mod help_edit;
mod home;
mod home_cards;
mod icons;
pub(crate) mod image;
mod item_details;
mod item_fam;
mod items;
pub(crate) mod kit;
mod library;
pub mod library_bulk;
mod library_pdfs;
pub mod library_search;
pub mod library_table;
mod md_editor;
mod menus;
mod modal;
mod modal_create;
mod modal_fields;
mod modal_found;
pub mod modal_import;
mod modal_size;
mod modal_transfer;
mod navbar;
pub mod notifications;
mod pdf;
mod pdf_arrange;
mod pdf_extract;
mod pdf_notes;
mod pdf_page;
mod pdf_paint;
mod pdf_side;
mod pdf_texture;
mod popup;
mod preview;
mod profiles;
mod quit;
mod reports;
mod reports_fam;
mod reports_runs;
mod run;
mod run_card;
mod run_timer;
mod session_form;
mod settings;
mod settings_replay;
mod setup;
mod setup_how;
mod setup_pacing;
mod setup_review;
mod setup_what;
mod sidebar;
mod stack_backlog_view;
mod stack_board_view;
mod stack_goals_view;
mod stack_reports_view;
mod stack_roadmap_view;
mod stack_summary_view;
mod stack_tree_view;
mod stack_view;
pub mod status_ui;
mod subjects;
mod summary;
mod summary_items;
mod toast;
mod tutorial;
mod welcome;
pub(crate) mod widgets;
pub(crate) mod window_frame;

use eframe::egui;

use crate::msg::Msg;
use crate::state::{Screen, State};

/// Draw the whole window for `state`.
pub fn draw(state: &State, ui: &mut egui::Ui) -> Vec<Msg> {
    let mut out = Vec::new();
    image::set_vault_root(ui.ctx(), state.vault.as_ref().map(|v| v.root.as_str()));
    window_frame::frame(ui);
    navbar::draw(state, ui, &mut out);
    sidebar::draw(state, ui, &mut out);
    bars::status(state, ui, &mut out);
    let page = egui::Frame::central_panel(ui.style()).inner_margin(egui::Margin::symmetric(20, 14));
    egui::CentralPanel::default()
        .frame(page)
        .show(ui, |ui| match state.screen {
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
            Screen::Help => help::draw(state, ui, &mut out),
        });
    if ui.input(|i| i.key_pressed(egui::Key::F1)) {
        out.push(Msg::Help(crate::help_types::HelpMsg::ForScreen));
    }
    tutorial::panel(state, ui, &mut out);
    toast::draw(state, ui, &mut out);
    modal::draw(state, ui, &mut out);
    quit::draw(state, ui, &mut out);
    out
}
