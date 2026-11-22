//! Edits to the open learn mode (STORY-0198): the graph's nodes and wires, the refusals after each edit, and
//! what a save asks for. Pure; see `learn_editor` for the state and the messages.

use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, check, reason};
use studyfrog_service::learn_mode_store::kind_name;

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

pub(crate) fn apply(e: &mut LearnEditor, msg: LearnMsg) -> Vec<LearnEffect> {
    match msg {
        LearnMsg::Open | LearnMsg::Start => {}
        LearnMsg::Listed(Ok(names)) => e.modes = names,
        LearnMsg::Listed(Err(why)) => e.notice = Some(format!("Could not list the modes: {why}")),
        LearnMsg::Pick(file) => return vec![LearnEffect::Load(file)],
        LearnMsg::Loaded(Ok((file, graph))) => {
            e.file = Some(file);
            e.graph = graph;
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
        LearnMsg::AddNode => {
            let id = fresh_id(&e.graph);
            let label = if e.label.trim().is_empty() {
                kind_name(e.kind).to_owned()
            } else {
                e.label.trim().to_owned()
            };
            e.graph.nodes.push(Node {
                id,
                kind: e.kind,
                label,
            });
            e.label.clear();
            refresh(e);
        }
        LearnMsg::RemoveNode(id) => {
            e.graph.nodes.retain(|n| n.id != id);
            e.graph.edges.retain(|w| w.from != id && w.to != id);
            refresh(e);
        }
        LearnMsg::From(id) => e.from = id,
        LearnMsg::To(id) => e.to = id,
        LearnMsg::When(when) => e.when = when,
        LearnMsg::AddWire => {
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
        LearnMsg::RemoveWire(i) => {
            if i < e.graph.edges.len() {
                e.graph.edges.remove(i);
                refresh(e);
            }
        }
        LearnMsg::Save => {
            if e.reasons.is_empty() {
                return vec![LearnEffect::Save(Box::new(e.graph.clone()))];
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
