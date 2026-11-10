//! The Console screen (STORY-0133, STORY-0148): interactive command REPL and
//! in-app ScriptStudio development environment with multi-line FrogScript editor.

use eframe::egui::{self, RichText, TextStyle};

use crate::console_ui::{ConsoleMsg, Entry, EntryKind};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

const HINT: &str =
    "Type a control command or Python script (e.g., `query type = card`, `state`, `logs`, `help`)";

const SCRIPT_TEMPLATES: &[(&str, &str)] = &[
    ("Query Due Items", "query type = card and due <= today\n"),
    (
        "Batch Create Cards",
        "item create card \"Bayes Theorem\" --stack \"Math\" --text \"What is Bayes theorem?\" --answer \"P(A|B) = P(B|A)P(A) / P(B)\"\n",
    ),
    ("Calendar & State Check", "state\nnav calendar\n"),
    (
        "Macro Syntax Test",
        "query type = note and text ~ \"{hide}\"\n",
    ),
];

fn entry(ui: &mut egui::Ui, e: &Entry) {
    let p = palette_of(ui);
    let (prefix, color) = match e.kind {
        EntryKind::Command => ("› ", p.accent),
        EntryKind::Answer => ("", p.text),
        EntryKind::Error => ("error: ", p.danger),
        EntryKind::Event => ("event ", p.text_weak),
        EntryKind::Script => ("", p.success),
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

#[allow(deprecated)]
fn script_editor(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let buffer_id = egui::Id::new("scriptstudio-code-buffer");
    let mut code = ui.data(|d| d.get_temp::<String>(buffer_id).unwrap_or_default());

    ui.horizontal(|ui| {
        ui.label(RichText::new("FrogScript Editor:").strong());
        egui::ComboBox::from_id_salt("script-template-picker")
            .selected_text("Load recipe template…")
            .show_ui(ui, |ui| {
                for &(name, snippet) in SCRIPT_TEMPLATES {
                    if ui.selectable_label(false, name).clicked() {
                        code = snippet.to_owned();
                        ui.data_mut(|d| d.insert_temp(buffer_id, code.clone()));
                    }
                }
            });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::primary_button(ui, "Run FrogScript").clicked() {
                for line in code.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        out.push(Msg::Console(ConsoleMsg::Edit(trimmed.to_owned())));
                        out.push(Msg::Console(ConsoleMsg::Submit));
                    }
                }
            }
            if kit::quiet_button(ui, "Clear Code").clicked() {
                code.clear();
                ui.data_mut(|d| d.insert_temp(buffer_id, String::new()));
            }
        });
    });

    let edit = egui::TextEdit::multiline(&mut code)
        .id_salt("scriptstudio-multiline-text")
        .font(TextStyle::Monospace)
        .desired_rows(5)
        .desired_width(f32::INFINITY);
    if ui.add(edit).changed() {
        ui.data_mut(|d| d.insert_temp(buffer_id, code));
    }
}

/// The Console screen.
#[allow(deprecated)]
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let editor_open_id = egui::Id::new("scriptstudio-editor-expanded");
    let mut editor_expanded = ui.data(|d| d.get_temp::<bool>(editor_open_id).unwrap_or(false));

    ui.horizontal(|ui| {
        ui.heading("Console");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::quiet_button(ui, "Clear").clicked() {
                out.push(Msg::Console(ConsoleMsg::Clear));
            }
            if kit::primary_button(ui, "Run").clicked() {
                out.push(Msg::Console(ConsoleMsg::Submit));
            }
            let label = if editor_expanded {
                "Hide ScriptStudio"
            } else {
                "ScriptStudio Editor"
            };
            if ui.button(label).clicked() {
                editor_expanded = !editor_expanded;
                ui.data_mut(|d| d.insert_temp(editor_open_id, editor_expanded));
            }
        });
    });

    let pal = palette_of(ui);
    ui.label(
        RichText::new("Commands go to this window's control protocol, as scripts use it; events and script output appear as they happen.")
            .color(pal.text_weak),
    );
    ui.add_space(6.0);

    singleline_input(state, ui, out);
    ui.add_space(6.0);

    if editor_expanded {
        script_editor(ui, out);
        ui.add_space(6.0);
    }

    kit::section(ui, "Output Log Stream");
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if state.console.entries.is_empty() {
                ui.label(
                    RichText::new("No command entries yet. Run a script above or type `help`.")
                        .color(pal.text_weak),
                );
            }
            for e in &state.console.entries {
                entry(ui, e);
                ui.add_space(2.0);
            }
        });
}
