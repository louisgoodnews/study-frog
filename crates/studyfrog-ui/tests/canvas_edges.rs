#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for FrogCanvas directed Bézier edge rendering, anchors, and previews (TASK-0256).

use eframe::egui::{Pos2, Rect};
use studyfrog_graph::canvas::{
    CanvasEdge, CanvasEdgeDir, CanvasNode, CanvasNodeKind, add_edge, add_node, new_canvas_doc,
};
use studyfrog_md::ulid::{format_ulid, parse_ulid};
use studyfrog_ui::canvas_types::{CanvasMsg, new_canvas_state};
use studyfrog_ui::canvas_ui::update_canvas;
use studyfrog_ui::view::canvas_edge::{AnchorSide, best_anchors, eval_cubic_bezier};

#[test]
fn test_anchor_minimization_side_by_side() {
    let r1 = Rect::from_min_size(Pos2::new(0.0, 0.0), eframe::egui::vec2(100.0, 50.0));
    let r2 = Rect::from_min_size(Pos2::new(300.0, 0.0), eframe::egui::vec2(100.0, 50.0));

    let (p1, side1, p2, side2) = best_anchors(r1, r2);
    assert_eq!(side1, AnchorSide::Right);
    assert_eq!(side2, AnchorSide::Left);
    assert_eq!(p1, Pos2::new(100.0, 25.0));
    assert_eq!(p2, Pos2::new(300.0, 25.0));
}

#[test]
fn test_anchor_minimization_stacked_vertically() {
    let r1 = Rect::from_min_size(Pos2::new(0.0, 0.0), eframe::egui::vec2(100.0, 50.0));
    let r2 = Rect::from_min_size(Pos2::new(0.0, 200.0), eframe::egui::vec2(100.0, 50.0));

    let (p1, side1, p2, side2) = best_anchors(r1, r2);
    assert_eq!(side1, AnchorSide::Bottom);
    assert_eq!(side2, AnchorSide::Top);
    assert_eq!(p1, Pos2::new(50.0, 50.0));
    assert_eq!(p2, Pos2::new(50.0, 200.0));
}

#[test]
fn test_cubic_bezier_endpoints_and_midpoint() {
    let p0 = Pos2::new(0.0, 0.0);
    let c1 = Pos2::new(10.0, 50.0);
    let c2 = Pos2::new(90.0, 50.0);
    let p1 = Pos2::new(100.0, 0.0);

    let start = eval_cubic_bezier(p0, c1, c2, p1, 0.0);
    assert!((start.x - p0.x).abs() < 1e-4);
    assert!((start.y - p0.y).abs() < 1e-4);

    let end = eval_cubic_bezier(p0, c1, c2, p1, 1.0);
    assert!((end.x - p1.x).abs() < 1e-4);
    assert!((end.y - p1.y).abs() < 1e-4);

    let mid = eval_cubic_bezier(p0, c1, c2, p1, 0.5);
    assert!((mid.x - 50.0).abs() < 1e-4);
    assert!(mid.y > 0.0);
}

#[test]
fn test_card_preview_open_and_close() {
    let test_uid = studyfrog_core::uid::Uid(0x018d_3456_789a_bcde_f012_3456_789a_bcde);
    let ulid_str = format_ulid(test_uid);

    let mut doc = new_canvas_doc();
    add_node(
        &mut doc,
        CanvasNode {
            id: "node-card-1".into(),
            kind: CanvasNodeKind::Card,
            item_id: Some(ulid_str.clone()),
            x: 50.0,
            y: 50.0,
            width: 180.0,
            height: 100.0,
            color: None,
            label: Some("Photosynthesis".into()),
        },
    );
    add_node(
        &mut doc,
        CanvasNode {
            id: "node-card-2".into(),
            kind: CanvasNodeKind::Card,
            item_id: None,
            x: 350.0,
            y: 50.0,
            width: 180.0,
            height: 100.0,
            color: None,
            label: Some("Chloroplast".into()),
        },
    );
    add_edge(
        &mut doc,
        CanvasEdge {
            id: "edge-1".into(),
            from: "node-card-1".into(),
            to: "node-card-2".into(),
            label: Some("occurs in".into()),
            direction: CanvasEdgeDir::Forward,
        },
    );

    let mut state = new_canvas_state(doc, "biology".into());
    assert!(state.preview.is_none());

    update_canvas(&mut state, CanvasMsg::OpenPreview("node-card-1".into()));
    assert!(state.preview.is_some());
    let prev = state.preview.as_ref().unwrap();
    assert_eq!(prev.title, "Photosynthesis");
    assert_eq!(prev.item_id, ulid_str);
    assert_eq!(parse_ulid(&prev.item_id), Some(test_uid));

    update_canvas(&mut state, CanvasMsg::ClosePreview);
    assert!(state.preview.is_none());
}
