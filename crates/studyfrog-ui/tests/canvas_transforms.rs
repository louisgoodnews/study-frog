#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for FrogCanvas 2D spatial coordinate transforms, zoom clamping, and node dragging (TASK-0255).

use studyfrog_graph::canvas::{CanvasNode, CanvasNodeKind, add_node, new_canvas_doc};
use studyfrog_graph::quadtree::new_rect;
use studyfrog_ui::canvas_types::{
    CanvasMsg, CanvasTransform, clamp_zoom, default_canvas_transform, new_canvas_state,
    screen_to_world, world_to_screen,
};
use studyfrog_ui::canvas_ui::{apply_pan, apply_zoom_at, clamp_coord, update_canvas};
use studyfrog_ui::view::canvas_paint::{build_node_quadtree, cull_visible_nodes};

#[test]
fn test_world_to_screen_and_back_roundtrip() {
    let tx = CanvasTransform {
        pan_x: 120.0,
        pan_y: -45.0,
        zoom: 1.5,
    };
    let center = (400.0, 300.0);
    let wx = 50.0;
    let wy = 80.0;

    let (sx, sy) = world_to_screen(&tx, wx, wy, center);
    let (back_wx, back_wy) = screen_to_world(&tx, sx, sy, center);

    assert!((wx - back_wx).abs() < 1e-5);
    assert!((wy - back_wy).abs() < 1e-5);
}

#[test]
fn test_clamp_zoom_bounds_and_nan() {
    assert_eq!(clamp_zoom(0.01), 0.05);
    assert_eq!(clamp_zoom(25.0), 20.0);
    assert_eq!(clamp_zoom(1.0), 1.0);
    assert_eq!(clamp_zoom(f64::NAN), 1.0);
    assert_eq!(clamp_zoom(f64::INFINITY), 1.0);
    assert_eq!(clamp_zoom(-2.0), 1.0);
}

#[test]
fn test_zoom_centered_at_cursor() {
    let mut tx = default_canvas_transform();
    let center = (500.0, 400.0);
    let cursor = (550.0, 420.0);

    let (w_before_x, w_before_y) = screen_to_world(&tx, cursor.0, cursor.1, center);
    apply_zoom_at(&mut tx, 2.0, cursor, center);
    assert_eq!(tx.zoom, 2.0);

    let (w_after_x, w_after_y) = screen_to_world(&tx, cursor.0, cursor.1, center);
    assert!((w_before_x - w_after_x).abs() < 1e-5);
    assert!((w_before_y - w_after_y).abs() < 1e-5);
}

#[test]
fn test_pan_delta() {
    let mut tx = CanvasTransform {
        pan_x: 0.0,
        pan_y: 0.0,
        zoom: 2.0,
    };
    apply_pan(&mut tx, 50.0, -30.0);
    assert_eq!(tx.pan_x, 25.0);
    assert_eq!(tx.pan_y, -15.0);
}

#[test]
fn test_node_dragging_updates_position() {
    let mut doc = new_canvas_doc();
    add_node(
        &mut doc,
        CanvasNode {
            id: "node-1".into(),
            kind: CanvasNodeKind::Card,
            item_id: Some("01JABCDEF".into()),
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 120.0,
            color: None,
            label: Some("Card Title".into()),
        },
    );

    let mut state = new_canvas_state(doc, "stack/math".into());
    assert_eq!(state.dragging, None);

    update_canvas(&mut state, CanvasMsg::DragStart("node-1".into()));
    assert!(state.dragging.is_some());
    assert_eq!(state.selected_node.as_deref(), Some("node-1"));

    update_canvas(&mut state, CanvasMsg::DragDelta(40.0, -20.0));
    assert_eq!(state.doc.nodes[0].x, 140.0);
    assert_eq!(state.doc.nodes[0].y, 80.0);
    assert!(state.dirty);

    update_canvas(&mut state, CanvasMsg::DragEnd);
    assert_eq!(state.dragging, None);
}

#[test]
fn test_quadtree_viewport_culling() {
    let nodes = vec![
        CanvasNode {
            id: "n1".into(),
            kind: CanvasNodeKind::Item,
            item_id: None,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            color: None,
            label: None,
        },
        CanvasNode {
            id: "n2".into(),
            kind: CanvasNodeKind::Item,
            item_id: None,
            x: 2000.0,
            y: 2000.0,
            width: 100.0,
            height: 50.0,
            color: None,
            label: None,
        },
    ];

    let tree = build_node_quadtree(&nodes);
    let view_rect = new_rect(-50.0, -50.0, 200.0, 200.0);
    let visible = cull_visible_nodes(&tree, &nodes, &view_rect);

    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, "n1");

    let empty_view = new_rect(0.0, 0.0, 0.0, 0.0);
    let culled_empty = cull_visible_nodes(&tree, &nodes, &empty_view);
    assert!(culled_empty.is_empty());
}

#[test]
fn test_clamp_coordinate_sanitization() {
    assert_eq!(clamp_coord(f64::NAN), 0.0);
    assert_eq!(clamp_coord(f64::INFINITY), 0.0);
    assert_eq!(clamp_coord(-f64::INFINITY), 0.0);
    assert_eq!(clamp_coord(50.0), 50.0);
}
