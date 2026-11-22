//! The Console screen (STORY-0133, STORY-0148): interactive command REPL and
//! in-app ScriptStudio multi-tab development environment with FrogScript execution.

use eframe::egui::{self, RichText, TextStyle};

pub mod code;
pub mod jobs;
pub mod output;

use crate::console_ui::{ConsoleMsg, Entry, EntryKind};
use crate::msg::Msg;
use crate::script_studio::active_tab;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

const HINT: &str = "Type a control command (e.g., `query type = card`, `state`, `logs`, `help`)";

fn entry(ui: &mut egui::Ui, e: &Entry) {
    let p = palette_of(ui);
    let (prefix, color) = match e.kind {
        EntryKind::Command => ("› ", p.accent),
        EntryKind::Answer => ("", p.text),
        EntryKind::Error => ("error: ", p.danger),
        EntryKind::Event => ("event ", p.text_weak),
        EntryKind::Script => ("+ ", p.success),
        EntryKind::Stdout => ("stdout: ", p.text),
        EntryKind::Stderr => ("stderr: ", p.danger),
    };
    // Each line carries its time on the app's clock, so a failure can be placed among the other lines.
    let text = RichText::new(format!("{:>8.1}s  {prefix}{}", e.at, e.text))
        .text_style(TextStyle::Monospace)
        .color(color);
    ui.add(egui::Label::new(text).wrap().selectable(true));
}

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

/// The editor, and the FrogScript reference beside it when it is open.
fn studio_editor(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if !state.console.studio.wiki.open {
        editor_column(state, ui, out);
        return;
    }
    let (w, h) = (ui.available_width(), ui.available_height());
    ui.horizontal_top(|ui| {
        ui.allocate_ui(egui::vec2(w * 0.62, h), |ui| editor_column(state, ui, out));
        ui.allocate_ui(egui::vec2(w * 0.36, h), |ui| {
            crate::view::studio_reference::draw(state, ui, out);
        });
    });
}

fn editor_column(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let studio = &state.console.studio;
    studio_tab_bar(state, ui, out);
    ui.add_space(4.0);

    let mut code = active_tab(studio).map_or("", |t| &t.content).to_string();
    code::show(ui, &mut code, out);
}

/// The Console and ScriptStudio screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let expanded = state.console.studio.expanded;

    ui.horizontal(|ui| {
        ui.heading("Console");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::primary_button(ui, "Run Command")
                .on_hover_text("Runs the command you typed.")
                .clicked()
            {
                out.push(Msg::Console(ConsoleMsg::Submit));
            }
            let label = if expanded {
                "Hide ScriptStudio"
            } else {
                "ScriptStudio Editor"
            };
            if ui
                .button(label)
                .on_hover_text("Shows or hides the ScriptStudio editor.")
                .clicked()
            {
                out.push(Msg::Console(ConsoleMsg::StudioToggleExpanded));
            }
        });
    });

    let pal = palette_of(ui);
    ui.label(
        RichText::new("Execute control commands or develop automated FrogScript pipelines with reactive triggers and dedicated stdout/stderr streams.")
            .color(pal.text_weak),
    );
    ui.add_space(6.0);

    singleline_input(state, ui, out);
    ui.add_space(6.0);

    if expanded {
        studio_editor(state, ui, out);
        ui.add_space(6.0);
    }

    output::bar(state, ui, out);
    ui.add_space(4.0);

    let visible_entries: Vec<_> = state
        .console
        .entries
        .iter()
        .filter(|e| super::console::output::visible(e, &state.console))
        .collect();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if visible_entries.is_empty() {
                ui.label(
                    RichText::new(
                        "No entries for selected filter. Type a command or run FrogScript above.",
                    )
                    .color(pal.text_weak),
                );
            }
            for e in visible_entries {
                entry(ui, e);
                ui.add_space(2.0);
            }
        });
}
