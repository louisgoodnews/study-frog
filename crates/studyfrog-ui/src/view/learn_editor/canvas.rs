//! The learn mode drawn as a canvas (STORY-0198): steps are boxes placed by their saved place, wires are arrows, and
//! a click or drag sends a message. The form below the canvas holds the same graph; nothing here changes it directly.

use eframe::egui::{self, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2, vec2};
use studyfrog_rehearsal::learn_graph::NodeKind;
use studyfrog_service::learn_mode_store::kind_name;

use crate::learn_editor::edit::pos_of;
use crate::learn_editor::{LearnEditor, LearnMsg};
use crate::msg::Msg;
use crate::theme::{palette_of, tint};

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
            .add_enabled(e.selected.is_some(), egui::Button::new("Delete selected"))
            .on_hover_text("Removes the selected step, with its wires.")
            .clicked()
        {
            send(out, LearnMsg::DeleteSelected);
        }
        let hint = if e.connecting {
            match e.from.is_empty() {
                true => "Click the step the wire starts from.".to_owned(),
                false => format!("From {}: click the step the wire ends at.", e.from),
            }
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

    for w in &e.graph.edges {
        if let (Some(a), Some(b)) = (rect_of(&w.from), rect_of(&w.to)) {
            wire(&painter, a, b, w.when.as_deref(), p.text_weak, caption);
        }
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

    let empty_at = back
        .interact_pointer_pos()
        .filter(|_| back.clicked())
        .filter(|at| !node_rects.iter().any(|(_, r)| r.contains(*at)));
    if let Some(at) = empty_at {
        send(out, LearnMsg::PlaceAt(at.x - rect.min.x, at.y - rect.min.y));
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

/// Where the segment from `center` towards `toward` leaves a box of half-size `half`.
fn edge_point(center: Pos2, toward: Pos2, half: Vec2) -> Pos2 {
    let d = toward - center;
    if d.length() < 1.0 {
        return center;
    }
    let tx = if d.x.abs() > 1e-3 {
        half.x / d.x.abs()
    } else {
        f32::INFINITY
    };
    let ty = if d.y.abs() > 1e-3 {
        half.y / d.y.abs()
    } else {
        f32::INFINITY
    };
    center + d * tx.min(ty).min(1.0)
}

/// An arrow from one box to another, with its condition written at its middle. A wire that goes back to a step on the
/// left is drawn as an arc above the boxes, so it does not run through the steps between them.
fn wire(
    painter: &egui::Painter,
    from: Rect,
    to: Rect,
    when: Option<&str>,
    color: Color32,
    caption: f32,
) {
    let half = NODE * 0.5;
    let (c1, c2) = (from.center(), to.center());
    if c1 == c2 {
        return;
    }
    let stroke = Stroke::new(1.5, color);
    if c2.x < c1.x - 1.0 {
        let top = from.top().min(to.top()) - 28.0;
        let (a, b) = (Pos2::new(c1.x, from.top()), Pos2::new(c2.x, to.top()));
        painter.add(egui::Shape::line(
            vec![a, Pos2::new(c1.x, top), Pos2::new(c2.x, top), b],
            stroke,
        ));
        arrow_head(painter, Pos2::new(c2.x, top), b, stroke);
        if let Some(w) = when {
            let at = Pos2::new((c1.x + c2.x) * 0.5, top - 2.0);
            painter.text(
                at,
                egui::Align2::CENTER_BOTTOM,
                w,
                FontId::proportional(caption),
                color,
            );
        }
        return;
    }
    let start = edge_point(c1, c2, half);
    let end = edge_point(c2, c1, half);
    painter.line_segment([start, end], stroke);
    arrow_head(painter, start, end, stroke);
    if let Some(w) = when {
        painter.text(
            (start + end.to_vec2()) * 0.5,
            egui::Align2::CENTER_CENTER,
            w,
            FontId::proportional(caption),
            color,
        );
    }
}

/// The two short strokes that make the head of an arrow pointing from `from` to `to`.
fn arrow_head(painter: &egui::Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    let u = (to - from).normalized();
    let side = vec2(-u.y, u.x) * 5.0;
    painter.line_segment([to, to - u * 10.0 + side], stroke);
    painter.line_segment([to, to - u * 10.0 - side], stroke);
}
