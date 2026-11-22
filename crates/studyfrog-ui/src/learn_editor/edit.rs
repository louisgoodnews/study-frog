//! Edits to the open learn mode (STORY-0198): the graph's nodes and wires, the canvas layout, the refusals after
//! each edit, and what a save asks for. Pure; see `learn_editor` for the state and the messages.

use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, check, reason};
use studyfrog_service::learn_mode_store::{Layout, kind_name};

use crate::learn_editor::{LearnEditor, LearnEffect, LearnMsg, new_editor};

/// Recompute the refusals after an edit.
fn refresh(e: &mut LearnEditor) {
    let mut reasons = Vec::new();
    if e.graph.name.trim().is_empty() {
        reasons.push("give the mode a name".to_owned());
    }
    reasons.extend(check(&e.graph).iter().map(reason));
    e.reasons = reasons;
}

/// The next node id that no node has yet: `n1`, `n2`, and so on.
fn fresh_id(g: &LearnGraph) -> String {
    (1..)
        .map(|k| format!("n{k}"))
        .find(|id| !g.nodes.iter().any(|n| &n.id == id))
        .unwrap_or_default()
}

/// Where a node is drawn: its saved place, or a grid place by its index when it has none yet.
pub fn pos_of(e: &LearnEditor, id: &str) -> [f32; 2] {
    if let Some(p) = e.layout.get(id) {
        return *p;
    }
    let index = e.graph.nodes.iter().position(|n| n.id == id).unwrap_or(0);
    let (col, row) = (index % 3, index / 3);
    [20.0 + col as f32 * 240.0, 30.0 + row as f32 * 110.0]
}

/// Add a step of the chosen kind, labelled as typed (or by its kind), at `at` when the canvas placed it.
fn add_node(e: &mut LearnEditor, at: Option<[f32; 2]>) {
    let id = fresh_id(&e.graph);
    let label = if e.label.trim().is_empty() {
        kind_name(e.kind).to_owned()
    } else {
        e.label.trim().to_owned()
    };
    if let Some(p) = at {
        e.layout.insert(id.clone(), p);
    }
    e.graph.nodes.push(Node {
        id,
        kind: e.kind,
        label,
    });
    e.label.clear();
    refresh(e);
}

/// Remove a step, its wires and its place.
fn remove_node(e: &mut LearnEditor, id: &str) {
    e.graph.nodes.retain(|n| n.id != id);
    e.graph.edges.retain(|w| w.from != id && w.to != id);
    e.layout.remove(id);
    if e.selected.as_deref() == Some(id) {
        e.selected = None;
    }
    refresh(e);
}

/// Add the wire from the chosen start to the chosen end, with the chosen condition.
fn add_wire(e: &mut LearnEditor) {
    if !e.from.is_empty() && !e.to.is_empty() {
        let when = (!e.when.is_empty()).then(|| e.when.clone());
        e.graph.edges.push(Edge {
            from: e.from.clone(),
            to: e.to.clone(),
            when,
        });
        refresh(e);
    }
}

/// The layout restricted to the steps the graph still has.
fn layout_of(e: &LearnEditor) -> Layout {
    e.layout
        .iter()
        .filter(|(id, _)| e.graph.nodes.iter().any(|n| &n.id == *id))
        .map(|(id, p)| (id.clone(), *p))
        .collect()
}

pub(crate) fn apply(e: &mut LearnEditor, msg: LearnMsg) -> Vec<LearnEffect> {
    match msg {
        LearnMsg::Open | LearnMsg::Start | LearnMsg::Started(_) => {}
        LearnMsg::ShowReport => match e.file.as_deref() {
            Some(file) => {
                return vec![LearnEffect::Report(
                    file.trim_end_matches(".toml").to_owned(),
                )];
            }
            None => e.notice = Some("Save the mode before its report can be read.".to_owned()),
        },
        LearnMsg::Reported(Ok(report)) => e.report = Some(report),
        LearnMsg::Reported(Err(why)) => {
            e.notice = Some(format!("Could not read the report: {why}"))
        }
        LearnMsg::Listed(Ok(names)) => e.modes = names,
        LearnMsg::Listed(Err(why)) => e.notice = Some(format!("Could not list the modes: {why}")),
        LearnMsg::Pick(file) => return vec![LearnEffect::Load(file)],
        LearnMsg::Loaded(Ok((file, graph, layout))) => {
            e.file = Some(file);
            e.graph = graph;
            e.layout = layout;
            e.selected = None;
            e.connecting = false;
            e.report = None;
            e.notice = None;
            refresh(e);
        }
        LearnMsg::Loaded(Err(why)) => e.notice = Some(format!("Could not open the mode: {why}")),
        LearnMsg::NewMode => {
            let modes = std::mem::take(&mut e.modes);
            *e = LearnEditor {
                modes,
                ..new_editor()
            };
        }
        LearnMsg::Name(name) => {
            e.graph.name = name;
            refresh(e);
        }
        LearnMsg::Kind(kind) => e.kind = kind,
        LearnMsg::Label(label) => e.label = label,
        LearnMsg::AddNode => add_node(e, None),
        LearnMsg::RemoveNode(id) => remove_node(e, &id),
        LearnMsg::From(id) => e.from = id,
        LearnMsg::To(id) => e.to = id,
        LearnMsg::When(when) => e.when = when,
        LearnMsg::AddWire => add_wire(e),
        LearnMsg::RemoveWire(i) => {
            if i < e.graph.edges.len() {
                e.graph.edges.remove(i);
                refresh(e);
            }
        }
        LearnMsg::ClickNode(id) => {
            if e.connecting {
                // The first click starts the wire, the second ends it; the condition is the one chosen in the form.
                if e.from.is_empty() {
                    e.from = id;
                } else {
                    e.to = id;
                    add_wire(e);
                    e.from.clear();
                    e.to.clear();
                }
            } else {
                e.selected = Some(id);
            }
        }
        LearnMsg::MoveBy(id, dx, dy) => {
            let [x, y] = pos_of(e, &id);
            e.layout.insert(id, [(x + dx).max(0.0), (y + dy).max(0.0)]);
        }
        LearnMsg::PlaceAt(x, y) => {
            if !e.connecting {
                add_node(e, Some([x.max(0.0), y.max(0.0)]));
            }
        }
        LearnMsg::ToggleConnect => {
            e.connecting = !e.connecting;
            e.from.clear();
            e.to.clear();
        }
        LearnMsg::DeleteSelected => {
            if let Some(id) = e.selected.clone() {
                remove_node(e, &id);
            }
        }
        LearnMsg::Save => {
            if e.reasons.is_empty() {
                return vec![LearnEffect::Save(Box::new(e.graph.clone()), layout_of(e))];
            }
            e.notice = Some("Not saved: fix the points listed below first.".to_owned());
        }
        LearnMsg::Saved(Ok(name)) => {
            e.notice = Some(format!("Saved as {name}."));
            e.file = Some(name);
            return vec![LearnEffect::List];
        }
        LearnMsg::Saved(Err(why)) => e.notice = Some(format!("Not saved: {why}")),
    }
    Vec::new()
}
