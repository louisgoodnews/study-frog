//! The Console headline (EPIC-073, STORY-0239): the title and the two run actions on the gradient panel. The same
//! actions, in the same words, with the same messages as before; only the drawing changes.

use eframe::egui::{self, RichText, vec2};

use crate::console_ui::ConsoleMsg;
use crate::msg::Msg;
use crate::theme::MIN_TARGET;
use crate::theme::style::Style;
use crate::view::gradient_panel;

/// The height of the headline panel, in points.
const HEADLINE_HEIGHT: f32 = 64.0;

/// The title, with "Run Command" and the ScriptStudio toggle on the right.
pub fn draw(expanded: bool, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    gradient_panel::panel(ui, HEADLINE_HEIGHT, |ui, s| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Console")
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                    .strong()
                    .color(s.gradient_text),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if action(ui, s, "Run Command", "Runs the command you typed.") {
                    out.push(Msg::Console(ConsoleMsg::Submit));
                }
                let label = if expanded {
                    "Hide ScriptStudio"
                } else {
                    "ScriptStudio Editor"
                };
                let tip = "Shows or hides the ScriptStudio editor.";
                if action(ui, s, label, tip) {
                    out.push(Msg::Console(ConsoleMsg::StudioToggleExpanded));
                }
            });
        });
    });
}

/// A button on the gradient: the headline's fill and text colours, with its tooltip. True when it is clicked.
fn action(ui: &mut egui::Ui, s: Style, text: &str, tip: &str) -> bool {
    ui.add(
        egui::Button::new(RichText::new(text).color(s.gradient_text))
            .fill(s.gradient[0])
            .min_size(vec2(0.0, MIN_TARGET)),
    )
    .on_hover_text(tip)
    .clicked()
}
