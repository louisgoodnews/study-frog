//! FrogCanvas 2D spatial concept board data model and TOML serializer.
//!
//! Plain data representations of canvas nodes, edges, frames, and documents
//! with TOML serialization and deserialization. Follows P1: zero inherent impl
//! blocks, domain logic is in free functions.

use serde::{Deserialize, Serialize};

/// Current FrogCanvas schema version.
pub const CURRENT_CANVAS_VERSION: u32 = 1;

fn default_canvas_version() -> u32 {
    CURRENT_CANVAS_VERSION
}

/// Type or role of a canvas node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CanvasNodeKind {
    /// Reference to a study item (e.g. flashcard or note).
    #[default]
    Item,
    /// Standalone markdown note node.
    Note,
    /// Flashcard preview or review node.
    Card,
    /// Freeform text label node.
    Text,
}

/// A node positioned on a 2D canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasNode {
    /// Unique node identifier.
    pub id: String,
    /// Node type or role.
    #[serde(default)]
    pub kind: CanvasNodeKind,
    /// Referenced item identifier (e.g. ULID), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    /// Horizontal position in canvas coordinate space.
    pub x: f64,
    /// Vertical position in canvas coordinate space.
    pub y: f64,
    /// Width in canvas units.
    pub width: f64,
    /// Height in canvas units.
    pub height: f64,
    /// Optional color override (e.g. hex code or color name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Optional text label or title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Directionality of a canvas edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CanvasEdgeDir {
    /// Directed edge pointing from source to target.
    #[default]
    Forward,
    /// Directed edge pointing from target to source.
    Backward,
    /// Bidirectional edge.
    Bi,
    /// Undirected connection.
    Undirected,
}

/// A connection between two nodes on the canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasEdge {
    /// Unique edge identifier.
    pub id: String,
    /// Source node identifier.
    pub from: String,
    /// Target node identifier.
    pub to: String,
    /// Optional edge label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Directionality of the edge.
    #[serde(default)]
    pub direction: CanvasEdgeDir,
}

/// A visual grouping frame or bounding boundary on the canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasFrame {
    /// Unique frame identifier.
    pub id: String,
    /// Title or label displayed on the frame boundary.
    pub title: String,
    /// Top-left horizontal position.
    pub x: f64,
    /// Top-left vertical position.
    pub y: f64,
    /// Frame width in canvas units.
    pub width: f64,
    /// Frame height in canvas units.
    pub height: f64,
    /// Optional color override for the frame border or background.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Top-level FrogCanvas document storing nodes, edges, and frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasDoc {
    /// Document schema version.
    #[serde(default = "default_canvas_version")]
    pub version: u32,
    /// List of spatial nodes.
    #[serde(default)]
    pub nodes: Vec<CanvasNode>,
    /// List of connections between nodes.
    #[serde(default)]
    pub edges: Vec<CanvasEdge>,
    /// List of visual grouping frames.
    #[serde(default)]
    pub frames: Vec<CanvasFrame>,
}

impl Default for CanvasDoc {
    fn default() -> Self {
        empty_canvas_doc()
    }
}

/// Returns an empty canvas document with the current schema version.
pub fn empty_canvas_doc() -> CanvasDoc {
    CanvasDoc {
        version: CURRENT_CANVAS_VERSION,
        nodes: Vec::new(),
        edges: Vec::new(),
        frames: Vec::new(),
    }
}

/// Creates a new canvas document with default settings.
pub fn new_canvas_doc() -> CanvasDoc {
    empty_canvas_doc()
}

/// Adds a node to the canvas document.
pub fn add_node(doc: &mut CanvasDoc, node: CanvasNode) {
    doc.nodes.push(node);
}

/// Adds an edge to the canvas document.
pub fn add_edge(doc: &mut CanvasDoc, edge: CanvasEdge) {
    doc.edges.push(edge);
}

/// Adds a frame to the canvas document.
pub fn add_frame(doc: &mut CanvasDoc, frame: CanvasFrame) {
    doc.frames.push(frame);
}

/// Finds a node by id.
pub fn find_node<'a>(doc: &'a CanvasDoc, id: &str) -> Option<&'a CanvasNode> {
    doc.nodes.iter().find(|n| n.id == id)
}

/// Removes a node by id and returns it, if found.
pub fn remove_node(doc: &mut CanvasDoc, id: &str) -> Option<CanvasNode> {
    let idx = doc.nodes.iter().position(|n| n.id == id)?;
    Some(doc.nodes.remove(idx))
}

/// Finds an edge by id.
pub fn find_edge<'a>(doc: &'a CanvasDoc, id: &str) -> Option<&'a CanvasEdge> {
    doc.edges.iter().find(|e| e.id == id)
}

/// Removes an edge by id and returns it, if found.
pub fn remove_edge(doc: &mut CanvasDoc, id: &str) -> Option<CanvasEdge> {
    let idx = doc.edges.iter().position(|e| e.id == id)?;
    Some(doc.edges.remove(idx))
}

/// Finds a frame by id.
pub fn find_frame<'a>(doc: &'a CanvasDoc, id: &str) -> Option<&'a CanvasFrame> {
    doc.frames.iter().find(|f| f.id == id)
}

/// Removes a frame by id and returns it, if found.
pub fn remove_frame(doc: &mut CanvasDoc, id: &str) -> Option<CanvasFrame> {
    let idx = doc.frames.iter().position(|f| f.id == id)?;
    Some(doc.frames.remove(idx))
}

/// Parse a CanvasDoc from a TOML string.
pub fn parse_canvas_toml(input: &str) -> Result<CanvasDoc, toml::de::Error> {
    toml::from_str(input)
}

/// Serialize a CanvasDoc to a TOML string.
pub fn serialize_canvas_toml(doc: &CanvasDoc) -> Result<String, toml::ser::Error> {
    toml::to_string(doc)
}
