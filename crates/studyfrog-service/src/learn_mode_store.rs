//! Learn modes stored in the vault as data (STORY-0198): one TOML file per mode in `learn_modes/`. A graph
//! that the rules refuse is not saved, and the reasons are returned.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, NodeKind, check, reason};

/// The folder of learn modes, inside the vault.
pub const FOLDER: &str = "learn_modes";

#[derive(Serialize, Deserialize)]
struct Doc {
    name: String,
    #[serde(default)]
    node: Vec<NodeDoc>,
    #[serde(default)]
    edge: Vec<EdgeDoc>,
}

#[derive(Serialize, Deserialize)]
struct NodeDoc {
    id: String,
    kind: String,
    #[serde(default)]
    label: String,
}

#[derive(Serialize, Deserialize)]
struct EdgeDoc {
    from: String,
    to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    when: Option<String>,
}

fn kind_name(k: NodeKind) -> &'static str {
    match k {
        NodeKind::Start => "start",
        NodeKind::Present => "present",
        NodeKind::Recall => "recall",
        NodeKind::Rate => "rate",
        NodeKind::Schedule => "schedule",
        NodeKind::Log => "log",
        NodeKind::Branch => "branch",
        NodeKind::Stop => "stop",
    }
}

fn kind_of(word: &str) -> Result<NodeKind, String> {
    Ok(match word {
        "start" => NodeKind::Start,
        "present" => NodeKind::Present,
        "recall" => NodeKind::Recall,
        "rate" => NodeKind::Rate,
        "schedule" => NodeKind::Schedule,
        "log" => NodeKind::Log,
        "branch" => NodeKind::Branch,
        "stop" => NodeKind::Stop,
        other => return Err(format!("unknown node kind {other}")),
    })
}

/// The file's text for `g`.
pub fn to_toml(g: &LearnGraph) -> Result<String, String> {
    let doc = Doc {
        name: g.name.clone(),
        node: g
            .nodes
            .iter()
            .map(|n| NodeDoc {
                id: n.id.clone(),
                kind: kind_name(n.kind).to_owned(),
                label: n.label.clone(),
            })
            .collect(),
        edge: g
            .edges
            .iter()
            .map(|e| EdgeDoc {
                from: e.from.clone(),
                to: e.to.clone(),
                when: e.when.clone(),
            })
            .collect(),
    };
    toml::to_string(&doc).map_err(|e| e.to_string())
}

/// The graph a file's text describes; an unknown node kind is an error.
pub fn from_toml(text: &str) -> Result<LearnGraph, String> {
    let doc: Doc = toml::from_str(text).map_err(|e| e.to_string())?;
    let nodes = doc
        .node
        .into_iter()
        .map(|n| {
            Ok(Node {
                kind: kind_of(&n.kind)?,
                id: n.id,
                label: n.label,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let edges = doc
        .edge
        .into_iter()
        .map(|e| Edge {
            from: e.from,
            to: e.to,
            when: e.when,
        })
        .collect();
    Ok(LearnGraph {
        name: doc.name,
        nodes,
        edges,
    })
}

/// The file name a mode of this name is stored under: lower-case letters, digits and dashes.
pub fn file_name(name: &str) -> String {
    let slug: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-');
    format!("{}.toml", if slug.is_empty() { "mode" } else { slug })
}

/// Save a valid graph into the vault's `learn_modes` folder; a graph the rules refuse is not saved.
pub fn save(root: &Path, g: &LearnGraph) -> Result<PathBuf, String> {
    let refusals = check(g);
    if !refusals.is_empty() {
        let why: Vec<String> = refusals.iter().map(reason).collect();
        return Err(format!("not saved: {}", why.join("; ")));
    }
    let dir = root.join(FOLDER);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(file_name(&g.name));
    fs::write(&path, to_toml(g)?).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Read the mode stored under `file` (its file name in `learn_modes`).
pub fn load(root: &Path, file: &str) -> Result<LearnGraph, String> {
    let text = fs::read_to_string(root.join(FOLDER).join(file)).map_err(|e| e.to_string())?;
    from_toml(&text)
}

/// The file names of the stored modes, sorted.
pub fn list(root: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(root.join(FOLDER)) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".toml"))
        .collect();
    names.sort();
    names
}
