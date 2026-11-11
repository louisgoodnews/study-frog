//! The Console screen (STORY-0133, STORY-0148): interactive command REPL and
//! in-app ScriptStudio multi-tab development environment with FrogScript execution.

use eframe::egui::{self, RichText, TextStyle};

use crate::console_ui::{ConsoleMsg, Entry, EntryKind};
use crate::msg::Msg;
use crate::script_studio::{ConsoleFilter, active_tab};
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
        EntryKind::Script => ("✓ ", p.success),
        EntryKind::Stdout => ("stdout: ", p.text),
        EntryKind::Stderr => ("stderr: ", p.danger),
    };
    let text = RichText::new(format!("{prefix}{}", e.text))
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
            if ui.small_button("×").clicked() {
                out.push(Msg::Console(ConsoleMsg::StudioCloseTab(i)));
            }
        }
        if ui.button("+ Tab").clicked() {
            out.push(Msg::Console(ConsoleMsg::StudioNewTab));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::primary_button(ui, "Run FrogScript").clicked() {
                out.push(Msg::Console(ConsoleMsg::StudioRunScript));
            }
            if kit::quiet_button(ui, "Save").clicked() {
                out.push(Msg::Console(ConsoleMsg::StudioSaveScript(None)));
            }
        });
    });
}

fn studio_editor(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let studio = &state.console.studio;
    studio_tab_bar(state, ui, out);
    ui.add_space(4.0);

    let mut code = active_tab(studio).map_or("", |t| &t.content).to_string();
    let edit = egui::TextEdit::multiline(&mut code)
        .id_salt("scriptstudio-multiline-text")
        .font(TextStyle::Monospace)
        .desired_rows(6)
        .desired_width(f32::INFINITY);
    let response = ui.add(edit);
    if response.changed() {
        out.push(Msg::Console(ConsoleMsg::StudioEditCode(code)));
    }
}

fn is_visible(e: &Entry, filter: ConsoleFilter) -> bool {
    match filter {
        ConsoleFilter::All => true,
        ConsoleFilter::Stdout => matches!(
            e.kind,
            EntryKind::Command | EntryKind::Answer | EntryKind::Script | EntryKind::Stdout
        ),
        ConsoleFilter::Stderr => matches!(e.kind, EntryKind::Error | EntryKind::Stderr),
        ConsoleFilter::Events => matches!(e.kind, EntryKind::Event),
    }
}

fn output_filter_bar(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let active = state.console.studio.filter;
    ui.horizontal(|ui| {
        ui.label(RichText::new("Streams:").strong());
        let filters = [
            (ConsoleFilter::All, "All"),
            (ConsoleFilter::Stdout, "Stdout"),
            (ConsoleFilter::Stderr, "Stderr / Errors"),
            (ConsoleFilter::Events, "Events"),
        ];
        for (f, name) in filters {
            if ui.selectable_label(active == f, name).clicked() {
                out.push(Msg::Console(ConsoleMsg::StudioSetFilter(f)));
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::quiet_button(ui, "Clear Output").clicked() {
                out.push(Msg::Console(ConsoleMsg::Clear));
            }
        });
    });
}

/// The Console and ScriptStudio screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let expanded = state.console.studio.expanded;

    ui.horizontal(|ui| {
        ui.heading("Console");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::primary_button(ui, "Run Command").clicked() {
                out.push(Msg::Console(ConsoleMsg::Submit));
            }
            let label = if expanded {
                "Hide ScriptStudio"
            } else {
                "ScriptStudio Editor"
            };
            if ui.button(label).clicked() {
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

    output_filter_bar(state, ui, out);
    ui.add_space(4.0);

    let filter = state.console.studio.filter;
    let visible_entries: Vec<_> = state
        .console
        .entries
        .iter()
        .filter(|e| is_visible(e, filter))
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
