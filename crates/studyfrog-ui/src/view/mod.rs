//! Views: plain functions from `&State` to the messages the user's input produced.

mod achievements;
mod agent;
mod agent_describe;
mod agent_exam;
mod agent_mcp;
mod agent_prompt_run;
mod agent_prompts;
mod agent_servers;
mod agent_staged;
mod agent_wiki;
mod backup;
mod bars;
mod board;
pub(crate) mod calendar;
mod calendar_grid;
mod calendar_reviews;
pub(crate) mod date_picker;
pub mod diary;
mod editor;
pub mod exam_card;
pub mod exam_result;
mod explorer;
mod explorer_charts;
pub mod extract_panel;
pub mod focus_bar;
pub mod folders;
mod graph;
mod help;
mod help_edit;
mod home;
mod home_cards;
mod icons;
pub(crate) mod image;
mod item_details;
mod item_fam;
pub mod item_stacks;
mod items;
pub(crate) mod kit;
mod library;
pub mod library_bulk;
pub mod library_keys;
mod library_pdfs;
pub mod library_search;
pub mod library_table;
mod md_editor;
mod menus;
mod modal;
mod modal_create;
mod modal_csv;
mod modal_fields;
mod modal_formats;
mod modal_found;
pub mod modal_import;
pub mod modal_pdf_export;
pub mod modal_pdf_preview;
pub mod modal_size;
mod modal_transfer;
mod modal_unmark;
mod navbar;
mod navbar_history;
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
mod preview_rich;
mod profiles;
mod prompts_screen;
mod report_cmrr;
mod report_pacer;
mod report_rows;
mod reports;
mod reports_calibration;
mod reports_fam;
mod reports_habits;
pub(crate) mod reports_leeches;
mod reports_runs;
mod reports_survival;
mod reports_workload;
mod run;
mod run_card;
mod run_diary_prompt;
mod run_slow;
mod run_timer;
mod session_form;
mod settings;
mod settings_appearance;
mod settings_files;
mod settings_replay;
mod setup;
mod setup_how;
mod setup_pacing;
mod setup_review;
mod setup_what;
mod sidebar;
mod stack_achievements;
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

use crate::appearance_ui::AppearanceMsg;
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
            Screen::Folders => folders::draw(state, ui, &mut out),
            Screen::Library => library::draw(state, ui, &mut out),
            Screen::Rehearse if state.rehearse.run.is_some() => run::draw(state, ui, &mut out),
            Screen::Rehearse => setup::draw(state, ui, &mut out),
            Screen::Settings => settings::draw(state, ui, &mut out),
            Screen::Editor => editor::draw(state, ui, &mut out),
            Screen::Graph => graph::draw(state, ui, &mut out),
            Screen::Calendar => calendar::draw(state, ui, &mut out),
            Screen::Board => board::draw(state, ui, &mut out),
            Screen::Achievements => achievements::draw(state, ui, &mut out),
            Screen::Agent => agent::draw(state, ui, &mut out),
            Screen::Prompts => prompts_screen::draw(state, ui, &mut out),
            Screen::Reports => reports::draw(state, ui, &mut out),
            Screen::Diary => diary::draw(state, ui, &mut out),
            Screen::Pdf => pdf::draw(state, ui, &mut out),
            Screen::Help => help::draw(state, ui, &mut out),
        });
    if ui.input(|i| i.key_pressed(egui::Key::F1)) {
        out.push(Msg::Help(crate::help_types::HelpMsg::ForScreen));
    }
    let zoom = |key| ui.input(|i| i.modifiers.command && !i.modifiers.alt && i.key_pressed(key));
    for (key, m) in [
        (egui::Key::Plus, AppearanceMsg::ZoomStep(1)),
        (egui::Key::Equals, AppearanceMsg::ZoomStep(1)),
        (egui::Key::Minus, AppearanceMsg::ZoomStep(-1)),
        (egui::Key::Num0, AppearanceMsg::ZoomReset),
    ] {
        if zoom(key) {
            out.push(Msg::Appearance(m));
        }
    }
    let alt = |key| ui.input(|i| i.modifiers.alt && i.key_pressed(key));
    for (key, m) in [
        (egui::Key::ArrowLeft, crate::history::HistoryMsg::Back),
        (egui::Key::ArrowRight, crate::history::HistoryMsg::Forward),
    ] {
        if alt(key) {
            out.push(Msg::Chrome(crate::chrome::ChromeMsg::History(m)));
        }
    }
    tutorial::panel(state, ui, &mut out);
    toast::draw(state, ui, &mut out);
    focus_bar::done_card(state, ui, &mut out);
    modal::draw(state, ui, &mut out);
    out
}
