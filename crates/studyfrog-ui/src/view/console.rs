//! The Console screen (STORY-0133): a transcript of commands, answers, script output and lifecycle
//! events, and an input line (Enter sends, Up/Down walk the history).

use eframe::egui::{self, RichText, TextStyle};

use crate::console_ui::{ConsoleMsg, Entry, EntryKind};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

const HINT: &str = "A control command (state, query type = card, item create note Title, nav reports, logs, help) or script run JOB|FILE, script list";

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

fn input(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
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

/// The screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.heading("Console");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::quiet_button(ui, "Clear").clicked() {
                out.push(Msg::Console(ConsoleMsg::Clear));
            }
            if kit::primary_button(ui, "Run").clicked() {
                out.push(Msg::Console(ConsoleMsg::Submit));
            }
        });
    });
    ui.label(
        RichText::new("Commands go to this window's control protocol, as scripts use it; events and script output appear as they happen.")
            .color(palette_of(ui).text_weak),
    );
    ui.add_space(8.0);
    input(state, ui, out);
    ui.add_space(8.0);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if state.console.entries.is_empty() {
                ui.label(
                    RichText::new("Nothing yet. Try `help` or `state`.")
                        .color(palette_of(ui).text_weak),
                );
            }
            for e in &state.console.entries {
                entry(ui, e);
                ui.add_space(4.0);
            }
        });
}
