//! The learn mode drawn as a canvas (STORY-0198): steps are boxes placed by their saved place, wires are arrows, and
//! a click or drag sends a message. The form below the canvas holds the same graph; nothing here changes it directly.

use eframe::egui::{self, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2, vec2};
use studyfrog_rehearsal::learn_graph::NodeKind;
use studyfrog_service::learn_mode_store::kind_name;

use crate::learn_editor::edit::pos_of;
use crate::learn_editor::{LearnEditor, LearnMsg};
use crate::msg::Msg;
use crate::theme::{palette_of, tint};

mod wires;

pub use wires::{distance_to_path, wire_path};
use wires::{wire, wire_at};

/// The height of the drawing area, in points.
pub const CANVAS_H: f32 = 300.0;
/// The size of a step's box, in points.
pub const NODE: Vec2 = vec2(170.0, 50.0);

fn send(out: &mut Vec<Msg>, m: LearnMsg) {
    out.push(Msg::Learn(m));
}

/// The toolbar and the drawing area.
pub fn draw(e: &LearnEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(egui::RichText::new("Canvas").strong());
    ui.horizontal(|ui| {
        if ui
            .selectable_label(e.connecting, "Connect")
            .on_hover_text("While this is on, click one step and then another to draw a wire between them. The condition is the one chosen in the form.")
            .clicked()
        {
            send(out, LearnMsg::ToggleConnect);
        }
        if ui
            .add_enabled(
                e.selected.is_some() || e.selected_wire.is_some(),
                egui::Button::new("Delete selected"),
            )
            .on_hover_text("Removes the selected wire, or else the selected step with its wires.")
            .clicked()
        {
            send(out, LearnMsg::DeleteSelected);
        }
        let hint = if e.connecting {
            match e.from.is_empty() {
                true => "Click the step the wire starts from.".to_owned(),
                false => format!("From {}: click the step the wire ends at.", e.from),
            }
        } else if e.selected_wire.is_some() {
            "The wire is selected. Delete selected removes it.".to_owned()
        } else {
            "Click empty space to add a step. Drag a step to move it.".to_owned()
        };
        ui.weak(hint);
    });
    let (rect, back) = ui.allocate_exact_size(vec2(ui.available_width(), CANVAS_H), Sense::click());
    let p = palette_of(ui);
    let caption = crate::typo::size(ui, crate::typo::Tier::Caption);
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        6.0,
        p.surface,
        Stroke::new(1.0, p.border),
        egui::StrokeKind::Inside,
    );

    let node_rects: Vec<(String, Rect)> = e
        .graph
        .nodes
        .iter()
        .map(|n| {
            let [x, y] = pos_of(e, &n.id);
            (
                n.id.clone(),
                Rect::from_min_size(rect.min + vec2(x, y), NODE),
            )
        })
        .collect();
    let rect_of = |id: &str| node_rects.iter().find(|(i, _)| i == id).map(|(_, r)| *r);

    // Each wire's path is worked out once: it is drawn here and tested for the click below.
    let paths: Vec<(usize, Vec<Pos2>)> = e
        .graph
        .edges
        .iter()
        .enumerate()
        .filter_map(|(i, w)| {
            let (a, b) = (rect_of(&w.from)?, rect_of(&w.to)?);
            Some((i, wire_path(a, b)))
        })
        .collect();
    for (i, path) in &paths {
        let w = &e.graph.edges[*i];
        let stroke = if e.selected_wire == Some(*i) {
            Stroke::new(3.0, p.accent)
        } else {
            Stroke::new(1.5, p.text_weak)
        };
        wire(&painter, path, w.when.as_deref(), stroke, caption);
    }
    for n in &e.graph.nodes {
        let Some(r) = rect_of(&n.id) else { continue };
        let resp = ui.interact(
            r,
            ui.scope_id().with(("learn-node", n.id.as_str())),
            Sense::click_and_drag(),
        );
        if resp.dragged() {
            let d = resp.drag_delta();
            send(out, LearnMsg::MoveBy(n.id.clone(), d.x, d.y));
        }
        if resp.clicked() {
            send(out, LearnMsg::ClickNode(n.id.clone()));
        }
        let color = kind_color(n.kind, &p);
        let selected = e.selected.as_deref() == Some(n.id.as_str());
        let stroke = if selected {
            Stroke::new(2.0, p.accent)
        } else {
            Stroke::new(1.0, color)
        };
        painter.rect(r, 8.0, tint(color, 40), stroke, egui::StrokeKind::Inside);
        let font = FontId::proportional(caption);
        painter.text(
            r.min + vec2(8.0, 6.0),
            egui::Align2::LEFT_TOP,
            format!("{}  {}", n.id, kind_name(n.kind)),
            font.clone(),
            p.text,
        );
        painter.text(
            r.min + vec2(8.0, 26.0),
            egui::Align2::LEFT_TOP,
            truncate(&n.label, 24),
            font,
            p.text_weak,
        );
    }

    // A click on a step was handled above; a click on a wire selects it; any other click adds a step.
    let click = back
        .interact_pointer_pos()
        .filter(|_| back.clicked())
        .filter(|at| !node_rects.iter().any(|(_, r)| r.contains(*at)));
    if let Some(at) = click {
        match wire_at(&paths, at) {
            Some(i) => send(out, LearnMsg::ClickWire(i)),
            None => send(out, LearnMsg::PlaceAt(at.x - rect.min.x, at.y - rect.min.y)),
        }
    }
}

fn kind_color(kind: NodeKind, p: &crate::theme::Palette) -> Color32 {
    match kind {
        NodeKind::Start => p.success,
        NodeKind::Stop => p.danger,
        NodeKind::Branch => p.warning,
        _ => p.accent,
    }
}

fn truncate(text: &str, n: usize) -> String {
    if text.chars().count() <= n {
        return text.to_owned();
    }
    let kept: String = text.chars().take(n - 1).collect();
    format!("{kept}…")
}
