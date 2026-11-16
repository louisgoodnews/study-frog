//! Interactive 2D spatial FrogCanvas view with pan, zoom, dragging, and culling.
//!
//! Follows Invariant P1: zero inherent impl blocks; logic in free functions.

use std::collections::HashMap;

use eframe::egui::{self, Pos2, Rect as EguiRect, Sense, Vec2};
use studyfrog_graph::quadtree::new_rect;

use crate::canvas_types::{CanvasMsg, CanvasState, screen_to_world, world_to_screen};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::{palette_of, tint};
use crate::view::canvas_edge::draw_edge;
use crate::view::canvas_paint::{
    build_node_quadtree, cull_visible_nodes, paint_frame, paint_grid, paint_node,
};
use crate::view::canvas_preview::draw_preview;

/// Header bar, zoom controls, interactive 2D canvas, and card preview popovers.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    let Some(canvas) = &state.canvas else {
        ui.label("FrogCanvas not initialized. Select a stack with a canvas to begin.");
        return;
    };

    draw_header(canvas, ui, out);
    ui.add_space(4.0);

    let height = (ui.available_height() - 20.0).max(250.0);
    let size = egui::vec2(ui.available_width(), height);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());

    handle_input(canvas, &resp, rect, ui, out);

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, p.background);

    paint_grid(&painter, &canvas.transform, rect, tint(p.border, 70));

    let center = (rect.center().x, rect.center().y);
    for frame in &canvas.doc.frames {
        paint_frame(&painter, frame, &canvas.transform, center, &p);
    }

    let (min_wx, min_wy) = screen_to_world(&canvas.transform, rect.min.x, rect.min.y, center);
    let (max_wx, max_wy) = screen_to_world(&canvas.transform, rect.max.x, rect.max.y, center);
    let view_rect = new_rect(
        min_wx,
        min_wy,
        (max_wx - min_wx).abs(),
        (max_wy - min_wy).abs(),
    );

    let tree = build_node_quadtree(&canvas.doc.nodes);
    let visible_nodes = cull_visible_nodes(&tree, &canvas.doc.nodes, &view_rect);

    let mut screen_rects = HashMap::new();
    for node in &canvas.doc.nodes {
        let (sx, sy) = world_to_screen(&canvas.transform, node.x, node.y, center);
        let sw = (node.width * canvas.transform.zoom) as f32;
        let sh = (node.height * canvas.transform.zoom) as f32;
        screen_rects.insert(
            &node.id,
            EguiRect::from_min_size(Pos2::new(sx, sy), Vec2::new(sw, sh)),
        );
    }

    for edge in &canvas.doc.edges {
        if let (Some(&from_r), Some(&to_r)) =
            (screen_rects.get(&edge.from), screen_rects.get(&edge.to))
        {
            let edge_color = p.text_weak;
            draw_edge(&painter, edge, from_r, to_r, edge_color, p.surface);
        }
    }

    for node in visible_nodes {
        let sel = canvas.selected_node.as_deref() == Some(&node.id);
        let hov = canvas.hovered_node.as_deref() == Some(&node.id);
        paint_node(&painter, node, &canvas.transform, center, (sel, hov), &p);
    }

    if let Some(preview) = &canvas.preview {
        draw_preview(ui.ctx(), preview, out);
    }
}

fn draw_header(canvas: &CanvasState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.heading("FrogCanvas");
        ui.add_space(8.0);
        let n = canvas.doc.nodes.len();
        let e = canvas.doc.edges.len();
        let f = canvas.doc.frames.len();
        ui.label(format!("{n} nodes · {e} edges · {f} frames"));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Reset View").clicked() {
                out.push(Msg::Canvas(CanvasMsg::ResetView));
            }
            if ui.button("－").clicked() {
                out.push(Msg::Canvas(CanvasMsg::Zoom(0.8)));
            }
            if ui.button("＋").clicked() {
                out.push(Msg::Canvas(CanvasMsg::Zoom(1.25)));
            }
            ui.label(format!("{:.0}%", canvas.transform.zoom * 100.0));
        });
    });
}

fn handle_input(
    canvas: &CanvasState,
    resp: &egui::Response,
    rect: EguiRect,
    ui: &egui::Ui,
    out: &mut Vec<Msg>,
) {
    let center = (rect.center().x, rect.center().y);
    if resp.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            let factor = (scroll * 0.002).exp();
            let cursor = resp.hover_pos().unwrap_or(rect.center());
            out.push(Msg::Canvas(CanvasMsg::ZoomAt {
                factor,
                center: (cursor.x, cursor.y),
            }));
        }
    }

    let cursor = resp.interact_pointer_pos();
    let hovered = cursor.and_then(|pos| {
        canvas.doc.nodes.iter().rev().find(|n| {
            let (sx, sy) = world_to_screen(&canvas.transform, n.x, n.y, center);
            let sw = (n.width * canvas.transform.zoom) as f32;
            let sh = (n.height * canvas.transform.zoom) as f32;
            EguiRect::from_min_size(Pos2::new(sx, sy), Vec2::new(sw, sh)).contains(pos)
        })
    });

    let hovered_id = hovered.map(|n| n.id.clone());
    if hovered_id != canvas.hovered_node {
        out.push(Msg::Canvas(CanvasMsg::Hover(hovered_id.clone())));
    }

    if resp.drag_started() {
        if let Some(target) = hovered_id {
            out.push(Msg::Canvas(CanvasMsg::DragStart(target)));
        }
    } else if resp.dragged() {
        let d = resp.drag_delta();
        if canvas.dragging.is_some() {
            out.push(Msg::Canvas(CanvasMsg::DragDelta(d.x, d.y)));
        } else {
            out.push(Msg::Canvas(CanvasMsg::Pan(d.x, d.y)));
        }
    } else if resp.drag_stopped() && canvas.dragging.is_some() {
        out.push(Msg::Canvas(CanvasMsg::DragEnd));
    }

    if resp.clicked() {
        let clicked_id = hovered.map(|n| n.id.clone());
        out.push(Msg::Canvas(CanvasMsg::Select(clicked_id)));
    }

    if resp.double_clicked()
        && let Some(n) = hovered
    {
        out.push(Msg::Canvas(CanvasMsg::OpenPreview(n.id.clone())));
    }
}
