//! The FrogCanvas headline and zoom toolbar in the redesign style (EPIC-073, STORY-0240): the title and the counts
//! on the gradient panel, the zoom controls in a card. The same buttons, messages and zoom readout as before.

use eframe::egui::{self, RichText};

use crate::canvas_types::{CanvasMsg, CanvasState};
use crate::msg::Msg;
use crate::theme::style::Style;
use crate::view::{gradient_panel, home_hero};

/// The height of the headline panel, in points.
const HEADLINE_HEIGHT: f32 = 64.0;

/// The hint shown while no canvas is open (the text the screen always had).
const EMPTY_HINT: &str = "FrogCanvas not initialized. Select a stack with a canvas to begin.";

/// The headline and the counts on the gradient panel, then the zoom controls in a card.
pub(super) fn draw(canvas: &CanvasState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let counts = counts_text(canvas);
    gradient_panel::panel(ui, HEADLINE_HEIGHT, |ui, s| headline(&counts, ui, s));
    ui.add_space(8.0);
    home_hero::card(ui, |ui| zoom_row(canvas, ui, out));
}

/// The same headline with no counts, and the hint for opening a canvas in a card (no canvas is open yet).
pub(super) fn draw_empty(ui: &mut egui::Ui) {
    gradient_panel::panel(ui, HEADLINE_HEIGHT, |ui, s| headline("", ui, s));
    ui.add_space(8.0);
    home_hero::card(ui, |ui| {
        ui.label(EMPTY_HINT);
    });
}

fn headline(counts: &str, ui: &mut egui::Ui, s: Style) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("FrogCanvas")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong()
                .color(s.gradient_text),
        );
        ui.label(RichText::new(counts).color(s.gradient_text));
    });
}

fn counts_text(canvas: &CanvasState) -> String {
    let n = canvas.doc.nodes.len();
    let e = canvas.doc.edges.len();
    let f = canvas.doc.frames.len();
    format!("{n} nodes · {e} edges · {f} frames")
}

/// The zoom controls, aligned to the right of the card, with the zoom readout last.
fn zoom_row(canvas: &CanvasState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.set_width(ui.available_width());
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui
            .button("Reset View")
            .on_hover_text("Puts the view back the way it was at the start.")
            .clicked()
        {
            out.push(Msg::Canvas(CanvasMsg::ResetView));
        }
        if ui.button("－").on_hover_text("Zooms out.").clicked() {
            out.push(Msg::Canvas(CanvasMsg::Zoom(0.8)));
        }
        if ui.button("＋").on_hover_text("Zooms in.").clicked() {
            out.push(Msg::Canvas(CanvasMsg::Zoom(1.25)));
        }
        ui.label(format!("{:.0}%", canvas.transform.zoom * 100.0));
    });
}
