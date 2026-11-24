#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The graph screen: pure update logic, the real-vault flow, and the real widgets.
use common::views::open_model;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_store::overview::{GraphNode, LinkGraph};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::effect::EffectEditor;
use studyfrog_ui::graph::{MAX_STEPS, ZOOM_RANGE};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::msg::MsgGraph;
use studyfrog_ui::msg::MsgVault;
use studyfrog_ui::state::Screen;

mod common;

fn chain(n: u128) -> LinkGraph {
    let nodes = (0..n)
        .map(|i| GraphNode {
            uid: Uid(i + 1),
            title: format!("Note {i}"),
            kind: ItemKind::Note,
        })
        .collect();
    let edges = (0..n.saturating_sub(1))
        .map(|i| (i as u32, i as u32 + 1))
        .collect();
    LinkGraph { nodes, edges }
}

fn loaded(graph: LinkGraph) -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Graph(MsgGraph::GraphLoaded(Ok(graph))));
    m
}

fn frame(m: &mut Model) {
    apply(
        m,
        Msg::Graph(MsgGraph::GraphFrame {
            width: 800.0,
            height: 600.0,
        }),
    );
}

#[test]
fn entering_the_screen_loads_the_graph_and_the_first_frame_fits_it() {
    let mut m = open_model();
    let fx = apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Graph)));
    assert_eq!(fx, [Effect::Editor(EffectEditor::LoadGraph)]);
    apply(&mut m, Msg::Graph(MsgGraph::GraphLoaded(Ok(chain(30)))));
    let g = m.state.graph.as_ref().unwrap();
    assert!(!g.fitted && !g.settled && g.steps == 0);
    frame(&mut m);
    let g = m.state.graph.as_ref().unwrap();
    assert!(g.fitted && g.steps > 0);
    assert!(
        g.sx.iter().any(|v| *v != 0.0),
        "screen positions were written"
    );
    // view_fit scales the whole square into the smaller canvas side (600 px): +-300 from the centre.
    assert!(
        g.sx.iter().chain(&g.sy).all(|v| v.abs() <= 301.0),
        "every node lies inside the canvas"
    );
}

#[test]
fn the_layout_settles_on_its_own_and_stops_simulating() {
    let mut m = loaded(chain(40));
    let mut frames = 0;
    while !m.state.graph.as_ref().unwrap().settled && frames < 5000 {
        frame(&mut m);
        frames += 1;
    }
    let g = m.state.graph.as_ref().unwrap();
    assert!(
        g.settled && g.steps < MAX_STEPS,
        "came to rest after {} steps",
        g.steps
    );
    let steps = g.steps;
    frame(&mut m);
    assert_eq!(
        m.state.graph.as_ref().unwrap().steps,
        steps,
        "no more steps once at rest"
    );
}

#[test]
fn pan_zoom_hover_and_restart() {
    let mut m = loaded(chain(10));
    frame(&mut m);
    let before = m.state.graph.as_ref().unwrap().view;
    apply(&mut m, Msg::Graph(MsgGraph::GraphPan(40.0, -20.0)));
    let after = m.state.graph.as_ref().unwrap().view;
    assert!(
        after.center_x < before.center_x && after.center_y > before.center_y,
        "dragging right moves the content right"
    );

    apply(&mut m, Msg::Graph(MsgGraph::GraphZoom(1e9)));
    assert_eq!(m.state.graph.as_ref().unwrap().view.zoom, ZOOM_RANGE.1);
    apply(&mut m, Msg::Graph(MsgGraph::GraphZoom(1e-18)));
    assert_eq!(m.state.graph.as_ref().unwrap().view.zoom, ZOOM_RANGE.0);

    apply(&mut m, Msg::Graph(MsgGraph::GraphHover(Some(3))));
    assert_eq!(m.state.graph.as_ref().unwrap().hovered, Some(3));
    let seed = m.state.graph.as_ref().unwrap().seed;
    apply(&mut m, Msg::Graph(MsgGraph::GraphRestart));
    let g = m.state.graph.as_ref().unwrap();
    assert!(g.seed != seed && !g.settled && g.steps == 0);
    assert_eq!(g.view.zoom, ZOOM_RANGE.0, "the view is kept");
}

#[test]
fn clicking_a_node_opens_it_and_a_bad_index_does_nothing() {
    let mut m = loaded(chain(5));
    assert_eq!(
        apply(&mut m, Msg::Graph(MsgGraph::GraphClick(2))),
        [Effect::Editor(EffectEditor::LoadDocument(Uid(3)))]
    );
    assert!(apply(&mut m, Msg::Graph(MsgGraph::GraphClick(99))).is_empty());
}

#[test]
fn a_refresh_keeps_known_positions_and_ignores_an_identical_graph() {
    let mut m = loaded(chain(6));
    for _ in 0..50 {
        frame(&mut m);
    }
    let old = m.state.graph.clone().unwrap();
    apply(&mut m, Msg::Graph(MsgGraph::GraphLoaded(Ok(chain(6)))));
    assert_eq!(
        m.state.graph.as_ref().unwrap(),
        &old,
        "an identical graph changes nothing"
    );

    apply(&mut m, Msg::Graph(MsgGraph::GraphLoaded(Ok(chain(7)))));
    let g = m.state.graph.as_ref().unwrap();
    assert_eq!(g.graph.nodes.len(), 7);
    for i in 0..6 {
        assert_eq!(
            (g.layout.x[i], g.layout.y[i]),
            (old.layout.x[i], old.layout.y[i]),
            "node {i} kept its place"
        );
    }
    assert!(!g.settled, "the new node needs to find its place");
}

#[test]
fn an_empty_graph_is_at_rest_and_a_failure_is_reported() {
    let mut m = loaded(LinkGraph::default());
    assert!(m.state.graph.as_ref().unwrap().settled);
    frame(&mut m);
    apply(&mut m, Msg::Graph(MsgGraph::GraphLoaded(Err("db".into()))));
    assert!(m.state.notice.as_ref().unwrap().text.contains("db"));
}
