#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for FrogCanvas data model and TOML serialization.

use studyfrog_graph::canvas::{
    CURRENT_CANVAS_VERSION, CanvasEdge, CanvasEdgeDir, CanvasFrame, CanvasNode, CanvasNodeKind,
    add_edge, add_frame, add_node, empty_canvas_doc, find_edge, find_frame, find_node,
    new_canvas_doc, parse_canvas_toml, remove_edge, remove_frame, remove_node,
    serialize_canvas_toml,
};

#[test]
fn empty_doc_defaults() {
    let doc = new_canvas_doc();
    assert_eq!(doc.version, CURRENT_CANVAS_VERSION);
    assert!(doc.nodes.is_empty());
    assert!(doc.edges.is_empty());
    assert!(doc.frames.is_empty());
}

#[test]
fn add_and_find_helpers() {
    let mut doc = new_canvas_doc();
    let node = CanvasNode {
        id: "n1".into(),
        kind: CanvasNodeKind::Card,
        item_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".into()),
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 80.0,
        color: Some("#ff0000".into()),
        label: Some("Card 1".into()),
    };
    add_node(&mut doc, node.clone());
    assert_eq!(doc.nodes.len(), 1);
    assert_eq!(find_node(&doc, "n1"), Some(&node));
    assert_eq!(find_node(&doc, "missing"), None);

    let edge = CanvasEdge {
        id: "e1".into(),
        from: "n1".into(),
        to: "n2".into(),
        label: Some("relates".into()),
        direction: CanvasEdgeDir::Forward,
    };
    add_edge(&mut doc, edge.clone());
    assert_eq!(find_edge(&doc, "e1"), Some(&edge));

    let frame = CanvasFrame {
        id: "f1".into(),
        title: "Group 1".into(),
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 300.0,
        color: None,
    };
    add_frame(&mut doc, frame.clone());
    assert_eq!(find_frame(&doc, "f1"), Some(&frame));
}

#[test]
fn remove_helpers() {
    let mut doc = new_canvas_doc();
    let node = CanvasNode {
        id: "n1".into(),
        kind: CanvasNodeKind::Card,
        item_id: None,
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 80.0,
        color: None,
        label: None,
    };
    add_node(&mut doc, node.clone());
    let edge = CanvasEdge {
        id: "e1".into(),
        from: "n1".into(),
        to: "n2".into(),
        label: None,
        direction: CanvasEdgeDir::Forward,
    };
    add_edge(&mut doc, edge.clone());
    let frame = CanvasFrame {
        id: "f1".into(),
        title: "Group 1".into(),
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 300.0,
        color: None,
    };
    add_frame(&mut doc, frame.clone());

    assert_eq!(remove_node(&mut doc, "n1"), Some(node));
    assert_eq!(find_node(&doc, "n1"), None);
    assert_eq!(remove_node(&mut doc, "n1"), None);

    assert_eq!(remove_edge(&mut doc, "e1"), Some(edge));
    assert_eq!(find_edge(&doc, "e1"), None);

    assert_eq!(remove_frame(&mut doc, "f1"), Some(frame));
    assert_eq!(find_frame(&doc, "f1"), None);
}

#[test]
fn roundtrip_serialization() {
    let mut doc = empty_canvas_doc();
    add_node(
        &mut doc,
        CanvasNode {
            id: "node_item".into(),
            kind: CanvasNodeKind::Item,
            item_id: Some("ulid123".into()),
            x: -50.5,
            y: 120.0,
            width: 200.0,
            height: 150.0,
            color: Some("#336699".into()),
            label: Some("Item Node".into()),
        },
    );
    add_node(
        &mut doc,
        CanvasNode {
            id: "node_text".into(),
            kind: CanvasNodeKind::Text,
            item_id: None,
            x: 250.0,
            y: 120.0,
            width: 180.0,
            height: 60.0,
            color: None,
            label: Some("Free text annotation".into()),
        },
    );
    add_edge(
        &mut doc,
        CanvasEdge {
            id: "edge_bi".into(),
            from: "node_item".into(),
            to: "node_text".into(),
            label: None,
            direction: CanvasEdgeDir::Bi,
        },
    );
    add_frame(
        &mut doc,
        CanvasFrame {
            id: "frame_main".into(),
            title: "Main Synthesis".into(),
            x: -100.0,
            y: 0.0,
            width: 600.0,
            height: 400.0,
            color: Some("#fafafa".into()),
        },
    );

    let serialized = serialize_canvas_toml(&doc).expect("serialization must succeed");
    let deserialized = parse_canvas_toml(&serialized).expect("deserialization must succeed");
    assert_eq!(doc, deserialized);
}

#[test]
fn parse_minimal_and_defaulted_toml() {
    let minimal = "version = 1\n";
    let doc = parse_canvas_toml(minimal).expect("minimal toml must parse");
    assert_eq!(doc.version, 1);
    assert!(doc.nodes.is_empty());
    assert!(doc.edges.is_empty());
    assert!(doc.frames.is_empty());

    let with_nodes = "version = 1\n\n[[nodes]]\nid = \"n1\"\nx = 0.0\ny = 0.0\nwidth = 100.0\nheight = 50.0\n\n[[edges]]\nid = \"e1\"\nfrom = \"n1\"\nto = \"n2\"\n";
    let parsed = parse_canvas_toml(with_nodes).expect("defaulted fields should parse");
    assert_eq!(parsed.nodes.len(), 1);
    assert_eq!(parsed.nodes[0].kind, CanvasNodeKind::Item);
    assert_eq!(parsed.nodes[0].item_id, None);
    assert_eq!(parsed.edges.len(), 1);
    assert_eq!(parsed.edges[0].direction, CanvasEdgeDir::Forward);
    assert_eq!(parsed.edges[0].label, None);
}

#[test]
fn node_kinds_and_edge_directions_parse_correctly() {
    let toml_str = r#"version = 1
nodes = [
  { id = "1", kind = "item", x = 0.0, y = 0.0, width = 10.0, height = 10.0 },
  { id = "2", kind = "note", x = 0.0, y = 0.0, width = 10.0, height = 10.0 },
  { id = "3", kind = "card", x = 0.0, y = 0.0, width = 10.0, height = 10.0 },
  { id = "4", kind = "text", x = 0.0, y = 0.0, width = 10.0, height = 10.0 }
]
edges = [
  { id = "e1", from = "1", to = "2", direction = "forward" },
  { id = "e2", from = "2", to = "3", direction = "backward" },
  { id = "e3", from = "3", to = "4", direction = "bi" },
  { id = "e4", from = "4", to = "1", direction = "undirected" }
]
"#;
    let doc = parse_canvas_toml(toml_str).expect("valid enum variants must parse");
    assert_eq!(doc.nodes[0].kind, CanvasNodeKind::Item);
    assert_eq!(doc.nodes[1].kind, CanvasNodeKind::Note);
    assert_eq!(doc.nodes[2].kind, CanvasNodeKind::Card);
    assert_eq!(doc.nodes[3].kind, CanvasNodeKind::Text);

    assert_eq!(doc.edges[0].direction, CanvasEdgeDir::Forward);
    assert_eq!(doc.edges[1].direction, CanvasEdgeDir::Backward);
    assert_eq!(doc.edges[2].direction, CanvasEdgeDir::Bi);
    assert_eq!(doc.edges[3].direction, CanvasEdgeDir::Undirected);
}

#[test]
fn invalid_toml_returns_error() {
    let invalid = "version = 'not_a_number'\n";
    let res = parse_canvas_toml(invalid);
    assert!(res.is_err());
}
