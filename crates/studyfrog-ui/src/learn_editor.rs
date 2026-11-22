//! The learn-mode editor (STORY-0198): a form to list the stored modes, open one, add and remove nodes and wires,
//! see why the mode cannot be saved yet, and save it. Pure: the files are read and written by effects.

use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, NodeKind, check, reason};
use studyfrog_service::learn_mode_store::kind_name;

use crate::effect::Effect;
use crate::modal::Modal;
use crate::state::State;

/// Every kind of node, in the order the form offers them.
pub const KINDS: [NodeKind; 8] = [
    NodeKind::Start,
    NodeKind::Present,
    NodeKind::Recall,
    NodeKind::Rate,
    NodeKind::Schedule,
    NodeKind::Log,
    NodeKind::Branch,
    NodeKind::Stop,
];

/// The conditions a branch wire may carry, in the order the form offers them.
pub const WHENS: [&str; 5] = ["again", "hard", "good", "easy", "default"];

/// The editor's fields: the stored modes, the open graph and its refusals, and the next node or wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnEditor {
    /// The stored modes (file names).
    pub modes: Vec<String>,
    /// The file the open graph was read from or saved to (`None` for a new mode).
    pub file: Option<String>,
    /// The graph being edited.
    pub graph: LearnGraph,
    /// The kind of the next node.
    pub kind: NodeKind,
    /// The label of the next node.
    pub label: String,
    /// The start of the next wire (a node id).
    pub from: String,
    /// The end of the next wire (a node id).
    pub to: String,
    /// The condition of the next wire (empty for a plain wire).
    pub when: String,
    /// Why the graph cannot be saved or run, in the order the check gives them (empty when it can).
    pub reasons: Vec<String>,
    /// The last thing that happened, for the form to say.
    pub notice: Option<String>,
}

/// An empty new mode.
pub fn new_editor() -> LearnEditor {
    LearnEditor {
        modes: Vec::new(),
        file: None,
        graph: LearnGraph::default(),
        kind: NodeKind::Present,
        label: String::new(),
        from: String::new(),
        to: String::new(),
        when: String::new(),
        reasons: vec![
            "give the mode a name".to_owned(),
            "the graph has no start node".to_owned(),
        ],
        notice: None,
    }
}

/// What the editor asks the executor to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearnEffect {
    /// Read the names of the stored modes.
    List,
    /// Read one stored mode.
    Load(String),
    /// Save this mode (only when it has no reasons to refuse).
    Save(Box<LearnGraph>),
}

/// What happened in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearnMsg {
    /// The editor was opened from the setup screen.
    Open,
    /// The stored modes were listed (or not).
    Listed(Result<Vec<String>, String>),
    /// A stored mode was chosen in the list.
    Pick(String),
    /// A stored mode was read (or not): its file name and graph.
    Loaded(Result<(String, LearnGraph), String>),
    /// "New mode" was pressed.
    NewMode,
    /// The mode's name was edited.
    Name(String),
    /// The kind of the next node was chosen.
    Kind(NodeKind),
    /// The label of the next node was edited.
    Label(String),
    /// "Add node" was pressed.
    AddNode,
    /// A node was removed, with its wires.
    RemoveNode(String),
    /// The start of the next wire was chosen.
    From(String),
    /// The end of the next wire was chosen.
    To(String),
    /// The condition of the next wire was chosen (empty for a plain wire).
    When(String),
    /// "Add wire" was pressed.
    AddWire,
    /// A wire was removed (by its place in the list).
    RemoveWire(usize),
    /// "Save" was pressed.
    Save,
    /// The mode was saved (under this file name), or not.
    Saved(Result<String, String>),
}

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

/// Apply a message to the editor in the open dialog, returning the effects it asks for.
pub fn update(state: State, msg: LearnMsg) -> (State, Vec<Effect>) {
    if msg == LearnMsg::Open {
        let mut state = state;
        state.modal = Some(Modal::LearnEditor(Box::new(new_editor())));
        return (state, vec![Effect::Learn(LearnEffect::List)]);
    }
    let mut state = state;
    let asked = match state.modal.as_mut() {
        Some(Modal::LearnEditor(e)) => apply(e, msg),
        _ => Vec::new(),
    };
    (state, asked.into_iter().map(Effect::Learn).collect())
}

fn apply(e: &mut LearnEditor, msg: LearnMsg) -> Vec<LearnEffect> {
    match msg {
        LearnMsg::Open => {}
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
