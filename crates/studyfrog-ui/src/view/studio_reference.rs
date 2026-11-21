//! The FrogScript reference beside ScriptStudio (STORY-0171): a search, the topics by category, the
//! picked topic's signature, return type, summary and example, and a button that adds the example to
//! the script.

use eframe::egui::{self, RichText, TextStyle};
use studyfrog_script::wiki::{CATEGORIES, Topic, label, search, topics};

use crate::console_ui::ConsoleMsg;
use crate::msg::Msg;
use crate::script_studio::active_tab;
use crate::script_studio::wiki_panel::WikiMsg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: WikiMsg) {
    out.push(Msg::Console(ConsoleMsg::Wiki(m)));
}

/// The key a topic is picked by: its category and name.
pub fn key_of(t: &Topic) -> String {
    format!("{}:{}", label(t.category), t.name)
}

fn picked(key: Option<&str>) -> Option<&'static Topic> {
    let key = key?;
    topics().into_iter().find(|t| key_of(t) == key)
}

/// Draw the reference panel.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let w = &state.console.studio.wiki;
    ui.heading("FrogScript reference");
    let mut query = w.query.clone();
    let field = egui::TextEdit::singleline(&mut query)
        .hint_text("Search name, type or category")
        .desired_width(f32::INFINITY);
    if ui.add(field).changed() {
        send(out, WikiMsg::Query(query.clone()));
    }
    ui.add_space(6.0);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if let Some(t) = picked(w.topic.as_deref()) {
                details(state, t, ui, out);
                ui.separator();
            }
            list(w.query.as_str(), w.topic.as_deref(), ui, out);
        });
}

fn list(query: &str, picked_key: Option<&str>, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let found = search(query);
    for category in CATEGORIES {
        let rows: Vec<&Topic> = found
            .iter()
            .copied()
            .filter(|t| t.category == category)
            .collect();
        if rows.is_empty() {
            continue;
        }
        ui.label(RichText::new(label(category)).strong());
        for t in rows {
            let key = key_of(t);
            if ui
                .selectable_label(picked_key == Some(key.as_str()), t.name)
                .clicked()
            {
                send(out, WikiMsg::Pick(key));
            }
        }
        ui.add_space(4.0);
    }
}

fn details(state: &State, t: &Topic, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    ui.label(
        RichText::new(t.name)
            .strong()
            .size(crate::typo::size(ui, crate::typo::Tier::Heading)),
    );
    ui.label(
        RichText::new(t.signature)
            .text_style(TextStyle::Monospace)
            .color(p.text_weak),
    );
    ui.label(format!("Returns: {}", t.returns));
    ui.label(t.summary);
    kit::card(ui, |ui| {
        ui.label(RichText::new(t.example).text_style(TextStyle::Monospace));
    });
    let add = kit::quiet_button(ui, "Add example to the script").on_hover_text(
        "Adds the example to the end of the script, where it can be run and changed.",
    );
    if add.clicked() {
        let mut code =
            active_tab(&state.console.studio).map_or_else(String::new, |tab| tab.content.clone());
        if !code.is_empty() && !code.ends_with('\n') {
            code.push('\n');
        }
        code.push_str(t.example);
        code.push('\n');
        out.push(Msg::Console(ConsoleMsg::StudioEditCode(code)));
    }
}
