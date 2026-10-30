//! The link graph screen: state and update logic. Pure; the layout engine is `studyfrog-graph`.

use studyfrog_core::uid::Uid;
use studyfrog_graph::layout::{Layout, layout_new, layout_step};
use studyfrog_graph::params::{Params, params_default};
use studyfrog_graph::view::{View, project, view_fit};
use studyfrog_store::overview::LinkGraph;

use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::{Notice, NoticeKind, State};

/// Layout steps per drawn frame while the graph is still moving.
pub const STEPS_PER_FRAME: u32 = 2;
/// Stop simulating after this many steps even if it never fully rests.
pub const MAX_STEPS: u32 = 4000;
/// Per-node kinetic energy below which the layout counts as at rest.
pub const REST_ENERGY: f32 = 0.01;
/// Zoom limits (screen pixels per layout unit).
pub const ZOOM_RANGE: (f32, f32) = (0.02, 40.0);

/// A graph on screen.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphState {
    /// Nodes and edges.
    pub graph: LinkGraph,
    /// Positions and velocities.
    pub layout: Layout,
    /// Simulation parameters.
    pub params: Params,
    /// Seed of the current layout.
    pub seed: u64,
    /// Pan and zoom.
    pub view: View,
    /// Screen x of each node relative to the canvas centre (written every frame).
    pub sx: Vec<f32>,
    /// Screen y of each node relative to the canvas centre.
    pub sy: Vec<f32>,
    /// Steps simulated so far.
    pub steps: u32,
    /// The layout has come to rest (or hit the step limit).
    pub settled: bool,
    /// The initial fit to the canvas has been done.
    pub fitted: bool,
    /// The node under the pointer.
    pub hovered: Option<usize>,
}

fn graph_state(graph: LinkGraph, seed: u64, previous: Option<&GraphState>) -> GraphState {
    let params = params_default();
    let n = graph.nodes.len();
    let mut layout = layout_new(n, seed, &params);
    if let Some(old) = previous {
        carry_positions(&mut layout, &graph, old);
    }
    let view = View {
        zoom: 1.0,
        center_x: 0.0,
        center_y: 0.0,
    };
    GraphState {
        graph,
        layout,
        params,
        seed,
        view,
        sx: vec![0.0; n],
        sy: vec![0.0; n],
        steps: 0,
        settled: n == 0,
        fitted: false,
        hovered: None,
    }
}

/// Nodes that were already on screen keep their position, so a refresh does not scramble the picture.
fn carry_positions(layout: &mut Layout, graph: &LinkGraph, old: &GraphState) {
    let index: std::collections::HashMap<Uid, usize> = old
        .graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.uid, i))
        .collect();
    for (i, node) in graph.nodes.iter().enumerate() {
        if let Some(&j) = index.get(&node.uid) {
            (layout.x[i], layout.y[i]) = (old.layout.x[j], old.layout.y[j]);
        }
    }
}

fn error(state: State, text: String) -> (State, Vec<Effect>) {
    (
        State {
            notice: Some(Notice {
                kind: NoticeKind::Error,
                text,
            }),
            ..state
        },
        Vec::new(),
    )
}

/// Handle the graph messages.
pub fn update(state: State, msg: Msg) -> (State, Vec<Effect>) {
    let mut state = state;
    match msg {
        Msg::GraphLoaded(Ok(graph)) => {
            let unchanged = state.graph.as_ref().is_some_and(|g| g.graph == graph);
            if !unchanged {
                state.graph = Some(graph_state(graph, 1, state.graph.as_ref()));
            }
            (state, Vec::new())
        }
        Msg::GraphLoaded(Err(e)) => error(state, format!("Could not load the graph: {e}")),
        Msg::GraphFrame { width, height } => (frame(state, width, height), Vec::new()),
        Msg::GraphPan(dx, dy) => (with_graph(state, |g| pan(g, dx, dy)), Vec::new()),
        Msg::GraphZoom(factor) => (with_graph(state, |g| zoom(g, factor)), Vec::new()),
        Msg::GraphHover(node) => (with_graph(state, |g| g.hovered = node), Vec::new()),
        Msg::GraphRestart => (with_graph(state, restart), Vec::new()),
        Msg::GraphClick(i) => click(state, i),
        _ => (state, Vec::new()),
    }
}

fn with_graph(mut state: State, f: impl FnOnce(&mut GraphState)) -> State {
    if let Some(g) = &mut state.graph {
        f(g);
    }
    state
}

fn frame(state: State, width: f32, height: f32) -> State {
    with_graph(state, |g| {
        if !g.fitted && width > 0.0 && height > 0.0 {
            g.view = view_fit(&g.layout, width, height);
            g.fitted = true;
        }
        if !g.settled {
            advance(g);
        }
        project(&g.layout, &g.view, 0.0, 0.0, &mut g.sx, &mut g.sy);
    })
}

fn advance(g: &mut GraphState) {
    let mut energy = f32::MAX;
    for _ in 0..STEPS_PER_FRAME {
        energy = layout_step(&mut g.layout, &g.graph.edges, &g.params);
        g.steps += 1;
    }
    let per_node = energy / g.graph.nodes.len().max(1) as f32;
    g.settled = per_node < REST_ENERGY || g.steps >= MAX_STEPS;
}

fn pan(g: &mut GraphState, dx: f32, dy: f32) {
    g.view.center_x -= dx / g.view.zoom;
    g.view.center_y -= dy / g.view.zoom;
}

fn zoom(g: &mut GraphState, factor: f32) {
    g.view.zoom = (g.view.zoom * factor).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
}

fn restart(g: &mut GraphState) {
    let fresh = graph_state(g.graph.clone(), g.seed.wrapping_add(1), None);
    let view = g.view;
    *g = GraphState {
        view,
        fitted: g.fitted,
        ..fresh
    };
}

fn click(state: State, index: usize) -> (State, Vec<Effect>) {
    let uid = state
        .graph
        .as_ref()
        .and_then(|g| g.graph.nodes.get(index))
        .map(|n| n.uid);
    match uid {
        Some(uid) => crate::editor::update(state, Msg::OpenItem(uid)),
        None => (state, Vec::new()),
    }
}
