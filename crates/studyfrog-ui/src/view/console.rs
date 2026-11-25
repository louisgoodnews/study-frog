//! The Console screen (STORY-0133, STORY-0148): interactive command REPL and
//! in-app ScriptStudio multi-tab development environment with FrogScript execution.

use eframe::egui::{self, RichText, TextStyle};

pub mod code;
pub mod headline;
pub mod jobs;
pub mod output;

use crate::console_ui::ConsoleMsg;
use crate::msg::Msg;
use crate::script_studio::active_tab;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::{home_hero, kit};

const HINT: &str = "Type a control command (e.g., `query type = card`, `state`, `logs`, `help`)";

const DESCRIPTION: &str = "Execute control commands or develop automated FrogScript pipelines with reactive triggers and dedicated stdout/stderr streams.";

fn singleline_input(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let c = &state.console;
    let mut text = c.input.clone();
    let field = egui::TextEdit::singleline(&mut text)
        .id_salt("console-input")
        .font(TextStyle::Monospace)
        .hint_text(HINT)
        .desired_width(f32::INFINITY);
    let response = ui.add(field);
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Console command"));
    if text != c.input {
        out.push(Msg::Console(ConsoleMsg::Edit(text)));
    }
    let key = |k| response.has_focus() && ui.input(|i| i.key_pressed(k));
    if key(egui::Key::ArrowUp) {
        out.push(Msg::Console(ConsoleMsg::Older));
    }
    if key(egui::Key::ArrowDown) {
        out.push(Msg::Console(ConsoleMsg::Newer));
    }
    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        out.push(Msg::Console(ConsoleMsg::Submit));
        response.request_focus();
    }
}

fn studio_tab_bar(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let studio = &state.console.studio;
    ui.horizontal_wrapped(|ui| {
        for (i, tab) in studio.tabs.iter().enumerate() {
            let label = if tab.dirty {
                format!("* {}", tab.name)
            } else {
                tab.name.clone()
            };
            let is_selected = i == studio.active_tab;
            if ui.selectable_label(is_selected, label).clicked() {
                out.push(Msg::Console(ConsoleMsg::StudioSelectTab(i)));
            }
            if ui.small_button("×").on_hover_text("Closes this.").clicked() {
                out.push(Msg::Console(ConsoleMsg::StudioCloseTab(i)));
            }
        }
        if ui.button("+ Tab").on_hover_text("Opens a new tab.").clicked() {
            out.push(Msg::Console(ConsoleMsg::StudioNewTab));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::primary_button(ui, "Run FrogScript").on_hover_text("Runs the script in the active tab.").clicked() {
                out.push(Msg::Console(ConsoleMsg::StudioRunScript));
            }
            if kit::quiet_button(ui, "Save").on_hover_text("Saves the script in the active tab.").clicked() {
                out.push(Msg::Console(ConsoleMsg::StudioSaveScript(None)));
            }
            let open = state.console.studio.wiki.open;
            let label = if open { "Hide reference" } else { "Reference" };
            let tip = "The FrogScript reference: syntax, operators, control flow, built-in functions and vault objects";
            if ui.button(label).on_hover_text(tip).clicked() {
                out.push(Msg::Console(ConsoleMsg::Wiki(
                    crate::script_studio::wiki_panel::WikiMsg::Toggle,
                )));
            }
        });
    });
}

/// The share of the height the editor and the reference take; the output pane below keeps the rest.
const STUDIO_SHARE: f32 = 0.6;
/// The least height of the editor and reference band, so the editor still shows a few lines.
const STUDIO_MIN_HEIGHT: f32 = 260.0;
/// The reference's share of the width when it opens, and the width range it can be resized within.
const REFERENCE_SHARE: f32 = 0.38;
const REFERENCE_MIN: f32 = 300.0;
/// The least width the editor keeps beside the reference.
const EDITOR_MIN: f32 = 300.0;

/// The editor, and the FrogScript reference beside it when it is open. The pair takes a share of the
/// height so the output pane below stays in the window. The reference is a resizable right panel and
/// the editor fills the rest of the width.
fn studio_editor(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if !state.console.studio.wiki.open {
        editor_column(state, ui, out);
        return;
    }
    let width = ui.available_width();
    let height = (ui.available_height() * STUDIO_SHARE).max(STUDIO_MIN_HEIGHT);
    let band = egui::vec2(width, height);
    ui.allocate_ui_with_layout(band, egui::Layout::top_down(egui::Align::Min), |ui| {
        egui::Panel::right("studio-reference")
            .default_size((width * REFERENCE_SHARE).max(REFERENCE_MIN))
            .size_range(REFERENCE_MIN..=(width - EDITOR_MIN).max(REFERENCE_MIN))
            .show(ui, |ui| crate::view::studio_reference::draw(state, ui, out));
        egui::CentralPanel::default().show(ui, |ui| editor_column(state, ui, out));
    });
}

fn editor_column(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let studio = &state.console.studio;
    studio_tab_bar(state, ui, out);
    ui.add_space(4.0);

    let mut code = active_tab(studio).map_or("", |t| &t.content).to_string();
    code::show(ui, &mut code, out);
}

/// The Console and ScriptStudio screen: the headline, the command on a card, the editor on a card when it is
/// open, and the output on a card that takes the rest of the window.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let expanded = state.console.studio.expanded;

    headline::draw(expanded, ui, out);
    ui.add_space(8.0);

    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(DESCRIPTION).color(palette_of(ui).text_weak));
        ui.add_space(6.0);
        singleline_input(state, ui, out);
    });
    ui.add_space(8.0);

    if expanded {
        home_hero::card(ui, |ui| {
            ui.set_width(ui.available_width());
            studio_editor(state, ui, out);
        });
        ui.add_space(8.0);
    }

    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        output::bar(state, ui, out);
        ui.add_space(4.0);
        output::list(state, ui);
    });
}
