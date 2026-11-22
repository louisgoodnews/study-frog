//! The learn-mode editor (STORY-0198): a form to list the stored modes, open one, add and remove nodes and wires,
//! see why the mode cannot be saved yet, and save it. Pure: the files are read and written by effects.

use studyfrog_rehearsal::learn_graph::{LearnGraph, NodeKind};
use studyfrog_service::learn_mode_store::Layout;

pub mod edit;

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
#[derive(Debug, Clone, PartialEq)]
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
    /// The custom report of the open mode, once asked for (opt-in; `None` until then).
    pub report: Option<studyfrog_store::learn_report::ModeReport>,
    /// Where each node is drawn on the canvas (a node without one is placed by its index).
    pub layout: Layout,
    /// The node the canvas last selected.
    pub selected: Option<String>,
    /// The wire the canvas last selected, by its index in the graph's wires.
    pub selected_wire: Option<usize>,
    /// Whether a click on a node starts or ends a wire (canvas connect mode).
    pub connecting: bool,
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
        report: None,
        layout: Layout::new(),
        selected: None,
        selected_wire: None,
        connecting: false,
    }
}

/// What the editor asks the executor to do.
#[derive(Debug, Clone, PartialEq)]
pub enum LearnEffect {
    /// Read the names of the stored modes.
    List,
    /// Read one stored mode.
    Load(String),
    /// Save this mode with its canvas layout (only when it has no reasons to refuse).
    Save(Box<LearnGraph>, Layout),
    /// Start a practice-only run of the saved mode with this file name.
    Start(String),
    /// Read the custom report of the mode with this id (its file name without `.toml`).
    Report(String),
}

/// What happened in the editor.
#[derive(Debug, Clone, PartialEq)]
pub enum LearnMsg {
    /// The editor was opened from the setup screen.
    Open,
    /// The stored modes were listed (or not).
    Listed(Result<Vec<String>, String>),
    /// A stored mode was chosen in the list.
    Pick(String),
    /// A stored mode was read (or not): its file name, graph and canvas layout.
    Loaded(Result<(String, LearnGraph, Layout), String>),
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
    /// "Start a run" was pressed: run the saved copy of the open mode.
    Start,
    /// "Show report" was pressed: read the custom report of the open mode.
    ShowReport,
    /// The custom report was read (or not).
    Reported(Result<studyfrog_store::learn_report::ModeReport, String>),
    /// A node was clicked on the canvas: selected, or joined to the wire being drawn in connect mode.
    ClickNode(String),
    /// A node was dragged on the canvas, by this many points.
    MoveBy(String, f32, f32),
    /// Empty canvas was clicked at this point: a step of the chosen kind is added there.
    PlaceAt(f32, f32),
    /// "Connect" was pressed: clicks on nodes now draw wires.
    ToggleConnect,
    /// A wire was clicked on the canvas (its index): selected.
    ClickWire(usize),
    /// "Delete selected" was pressed: the selected wire goes, or else the selected node with its wires.
    DeleteSelected,
    /// The run of a learn mode was made, with its graph (or why not).
    Started(Result<(studyfrog_core::run::Run, LearnGraph), String>),
}

/// Start the open mode: only a saved mode with no reasons can run. The dialog closes; the run view takes over.
fn start(state: State) -> (State, Vec<Effect>) {
    let mut state = state;
    let file = match state.modal.as_ref() {
        Some(Modal::LearnEditor(e)) if e.reasons.is_empty() => e.file.clone(),
        _ => None,
    };
    match file {
        Some(file) => {
            state.modal = None;
            state.rehearse.starting = true;
            (state, vec![Effect::Learn(LearnEffect::Start(file))])
        }
        None => {
            if let Some(Modal::LearnEditor(e)) = state.modal.as_mut() {
                e.notice = Some("Save the mode before it can run.".to_owned());
            }
            (state, Vec::new())
        }
    }
}

/// Apply a message to the editor in the open dialog, returning the effects it asks for.
pub fn update(state: State, msg: LearnMsg) -> (State, Vec<Effect>) {
    if msg == LearnMsg::Open {
        let mut state = state;
        state.modal = Some(Modal::LearnEditor(Box::new(new_editor())));
        return (state, vec![Effect::Learn(LearnEffect::List)]);
    }
    if msg == LearnMsg::Start {
        return start(state);
    }
    if let LearnMsg::Started(result) = msg {
        return match result {
            Ok((run, graph)) => crate::run_update::begin_run(state, run, Some(graph)),
            Err(why) => {
                let mut state = state;
                state.rehearse.starting = false;
                crate::run_update::error(state, format!("Could not start the run: {why}"))
            }
        };
    }
    let mut state = state;
    let asked = match state.modal.as_mut() {
        Some(Modal::LearnEditor(e)) => edit::apply(e, msg),
        _ => Vec::new(),
    };
    (state, asked.into_iter().map(Effect::Learn).collect())
}
