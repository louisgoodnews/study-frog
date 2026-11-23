//! The answers of `ui tree`, `ui find` and `ui click`: the named widgets of a frame's AccessKit tree,
//! the widgets a label finds, and the request that clicks one. Pure.

use std::collections::{HashMap, HashSet};

use eframe::egui::accesskit::{Action, ActionRequest, Node, NodeId, TreeId, TreeUpdate};
use serde_json::{Value, json};

/// Widgets a `ui tree` answer lists at most.
pub const MAX_WIDGETS: usize = 300;
/// Widgets a `ui find` answer lists at most.
const MAX_MATCHES: usize = 20;

/// One named widget of the frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Widget {
    /// The node, for a click.
    pub id: NodeId,
    /// The role (`Button`, `TextInput`, ...).
    pub role: String,
    /// The label (may be empty when the widget shows a value).
    pub label: String,
    /// The value, if the widget has one.
    pub value: Option<String>,
    /// Whether the widget takes input.
    pub enabled: bool,
}

fn widget_of(id: NodeId, node: &Node) -> Widget {
    Widget {
        id,
        role: format!("{:?}", node.role()),
        label: node.label().unwrap_or_default().to_owned(),
        value: node.value().map(str::to_owned),
        enabled: !node.is_disabled(),
    }
}

/// Push the children of `id` (in reverse, so the first child is visited first).
fn push_children(by_id: &HashMap<NodeId, &Node>, id: NodeId, todo: &mut Vec<NodeId>) {
    if let Some(node) = by_id.get(&id) {
        todo.extend(node.children().iter().rev().copied());
    }
}

/// The nodes of the tree from its root, in reading order (depth first); nodes not under the root
/// are not in the tree.
fn in_order(update: &TreeUpdate) -> Vec<(NodeId, &Node)> {
    let by_id: HashMap<NodeId, &Node> = update.nodes.iter().map(|(id, n)| (*id, n)).collect();
    let root = update.tree.as_ref().map(|t| t.root);
    let mut todo: Vec<NodeId> = root.into_iter().collect();
    let mut seen: HashSet<NodeId> = HashSet::new();
    let mut order = Vec::new();
    while let Some(id) = todo.pop() {
        if !seen.insert(id) {
            continue;
        }
        if let Some(node) = by_id.get(&id) {
            order.push((id, *node));
        }
        push_children(&by_id, id, &mut todo);
    }
    order
}

/// The named widgets of the frame: those with a label or a value.
pub fn widgets(update: &TreeUpdate) -> Vec<Widget> {
    in_order(update)
        .into_iter()
        .map(|(id, node)| widget_of(id, node))
        .filter(|w| !w.label.is_empty() || w.value.is_some())
        .collect()
}

fn widget_json(w: &Widget) -> Value {
    json!({
        "role": w.role, "label": w.label, "value": w.value, "enabled": w.enabled,
    })
}

/// The answer of `ui tree`: every named widget, at most [`MAX_WIDGETS`].
pub fn tree_json(update: &TreeUpdate) -> Value {
    let all = widgets(update);
    let shown: Vec<Value> = all.iter().take(MAX_WIDGETS).map(widget_json).collect();
    json!({
        "count": all.len(), "truncated": all.len() > MAX_WIDGETS, "widgets": shown,
    })
}

/// The widgets labelled `label`: exactly (ignoring case), or else containing it.
pub fn find(update: &TreeUpdate, label: &str) -> Vec<Widget> {
    let want = label.trim().to_lowercase();
    let named = widgets(update);
    let exact: Vec<Widget> = named
        .iter()
        .filter(|w| w.label.to_lowercase() == want)
        .cloned()
        .collect();
    if !exact.is_empty() {
        return exact;
    }
    named
        .into_iter()
        .filter(|w| w.label.to_lowercase().contains(&want))
        .collect()
}

/// The answer of `ui find`; no widget is an error that names the label.
pub fn find_json(update: &TreeUpdate, label: &str) -> Result<Value, String> {
    let found = find(update, label);
    if found.is_empty() {
        return Err(format!(
            "no widget labelled {} (ui tree lists the names)",
            label.trim()
        ));
    }
    let shown: Vec<Value> = found.iter().take(MAX_MATCHES).map(widget_json).collect();
    Ok(json!({"label": label.trim(), "count": found.len(), "matches": shown}))
}

/// The one widget a click goes to. Several matches are an error (the caller names the exact label).
pub fn pick(update: &TreeUpdate, label: &str) -> Result<Widget, String> {
    let mut found = find(update, label);
    match found.len() {
        0 => Err(format!(
            "no widget labelled {} (ui tree lists the names)",
            label.trim()
        )),
        1 => Ok(found.remove(0)),
        n => Err(format!(
            "{n} widgets are labelled {}: give the exact label (ui tree lists them)",
            label.trim()
        )),
    }
}

/// The request that clicks `id` in the root tree.
pub fn click_request(id: NodeId) -> ActionRequest {
    ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None,
    }
}
