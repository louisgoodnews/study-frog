//! FrogCanvas state update and manipulation logic.
//!
//! Follows Invariant P1: zero inherent impl blocks; all domain logic in free functions.

use crate::canvas_types::{
    CanvasMsg, CanvasState, CanvasTransform, DraggingState, clamp_zoom, screen_to_world,
};
use crate::effect::Effect;
use crate::state::State;

const MAX_COORD: f64 = 100_000.0;

/// Clamps coordinate values to finite bounds, mapping NaN / inf to 0.0.
pub fn clamp_coord(val: f64) -> f64 {
    if !val.is_finite() {
        0.0
    } else {
        val.clamp(-MAX_COORD, MAX_COORD)
    }
}

/// Applies panning delta in screen pixels to the canvas transform.
pub fn apply_pan(tx: &mut CanvasTransform, dx: f32, dy: f32) {
    let z = if tx.zoom.abs() < 1e-6 { 1.0 } else { tx.zoom };
    tx.pan_x = clamp_coord(tx.pan_x + (dx as f64 / z));
    tx.pan_y = clamp_coord(tx.pan_y + (dy as f64 / z));
}

/// Multiplies current zoom by factor, clamping within allowed bounds.
pub fn apply_zoom(tx: &mut CanvasTransform, factor: f32) {
    let f = if !factor.is_finite() || factor <= 0.0 {
        1.0
    } else {
        factor as f64
    };
    tx.zoom = clamp_zoom(tx.zoom * f);
}

/// Zooms centered on a specific screen position `(sx, sy)`.
pub fn apply_zoom_at(
    tx: &mut CanvasTransform,
    factor: f32,
    screen_pos: (f32, f32),
    center: (f32, f32),
) {
    let f = if !factor.is_finite() || factor <= 0.0 {
        1.0
    } else {
        factor as f64
    };
    let new_zoom = clamp_zoom(tx.zoom * f);
    let (wx, wy) = screen_to_world(tx, screen_pos.0, screen_pos.1, center);
    tx.zoom = new_zoom;
    tx.pan_x = clamp_coord(((screen_pos.0 - center.0) as f64 / new_zoom) - wx);
    tx.pan_y = clamp_coord(((screen_pos.1 - center.1) as f64 / new_zoom) - wy);
}

/// Starts dragging the node with the given ID.
pub fn start_drag(canvas: &mut CanvasState, node_id: String) {
    if let Some(node) = canvas.doc.nodes.iter().find(|n| n.id == node_id) {
        canvas.dragging = Some(DraggingState {
            node_id: node.id.clone(),
            start_x: node.x,
            start_y: node.y,
        });
        canvas.selected_node = Some(node.id.clone());
    }
}

/// Moves the currently dragged node by screen delta `(dx, dy)`.
pub fn move_dragged_node(canvas: &mut CanvasState, dx: f32, dy: f32) {
    let Some(dragging) = &canvas.dragging else {
        return;
    };
    let target_id = dragging.node_id.clone();
    let z = if canvas.transform.zoom.abs() < 1e-6 {
        1.0
    } else {
        canvas.transform.zoom
    };
    let world_dx = dx as f64 / z;
    let world_dy = dy as f64 / z;

    if let Some(node) = canvas.doc.nodes.iter_mut().find(|n| n.id == target_id) {
        node.x = clamp_coord(node.x + world_dx);
        node.y = clamp_coord(node.y + world_dy);
        canvas.dirty = true;
    }
}

/// Ends the current node dragging operation.
pub fn end_drag(canvas: &mut CanvasState) {
    canvas.dragging = None;
}

/// Opens card preview popover for the specified node id.
pub fn open_node_preview(canvas: &mut CanvasState, node_id: &str) {
    if let Some(node) = canvas.doc.nodes.iter().find(|n| n.id == node_id) {
        let item_id = node.item_id.clone().unwrap_or_else(|| node.id.clone());
        let title = node
            .label
            .clone()
            .unwrap_or_else(|| format!("Node {}", &node.id[..node.id.len().min(8)]));
        let content = format!(
            "**Type**: {:?}\n\n**Item ID**: `{}`\n\n**Position**: ({:.1}, {:.1})",
            node.kind, item_id, node.x, node.y
        );
        canvas.preview = Some(crate::canvas_types::CardPreviewState {
            node_id: node.id.clone(),
            item_id,
            title,
            content,
        });
    }
}

/// Advances canvas state in response to a CanvasMsg.
pub fn update_canvas(canvas: &mut CanvasState, msg: CanvasMsg) {
    match msg {
        CanvasMsg::Pan(dx, dy) => apply_pan(&mut canvas.transform, dx, dy),
        CanvasMsg::Zoom(factor) => apply_zoom(&mut canvas.transform, factor),
        CanvasMsg::ZoomAt { factor, center } => {
            apply_zoom_at(&mut canvas.transform, factor, center, center);
        }
        CanvasMsg::ResetView => {
            canvas.transform = crate::canvas_types::default_canvas_transform();
        }
        CanvasMsg::DragStart(id) => start_drag(canvas, id),
        CanvasMsg::DragDelta(dx, dy) => move_dragged_node(canvas, dx, dy),
        CanvasMsg::DragEnd => end_drag(canvas),
        CanvasMsg::Select(id) => canvas.selected_node = id,
        CanvasMsg::Hover(id) => canvas.hovered_node = id,
        CanvasMsg::OpenPreview(id) => open_node_preview(canvas, &id),
        CanvasMsg::ClosePreview => canvas.preview = None,
        CanvasMsg::SetDoc(doc) => {
            canvas.doc = doc;
            canvas.dirty = false;
        }
    }
}

/// Pure top-level update step for `Msg::Canvas`.
pub fn update(mut state: State, msg: CanvasMsg) -> (State, Vec<Effect>) {
    if let Some(canvas) = &mut state.canvas {
        update_canvas(canvas, msg);
    }
    (state, Vec::new())
}
