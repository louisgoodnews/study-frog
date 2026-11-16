//! FrogCanvas plain data types and coordinate transformation functions.
//!
//! Follows Invariant P1: zero inherent impl blocks; all logic in free functions.

use studyfrog_graph::canvas::CanvasDoc;

/// Minimum zoom factor allowed on canvas.
pub const MIN_ZOOM: f64 = 0.05;
/// Maximum zoom factor allowed on canvas.
pub const MAX_ZOOM: f64 = 20.0;

/// 2D affine canvas transform representing panning offset and scale zoom.
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasTransform {
    /// Horizontal pan offset in canvas world units.
    pub pan_x: f64,
    /// Vertical pan offset in canvas world units.
    pub pan_y: f64,
    /// Scaling zoom factor clamped to `[MIN_ZOOM, MAX_ZOOM]`.
    pub zoom: f64,
}

impl Default for CanvasTransform {
    fn default() -> Self {
        default_canvas_transform()
    }
}

/// Active drag state for interactive node repositioning.
#[derive(Debug, Clone, PartialEq)]
pub struct DraggingState {
    /// Unique identifier of the node being dragged.
    pub node_id: String,
    /// Initial node horizontal coordinate at drag start.
    pub start_x: f64,
    /// Initial node vertical coordinate at drag start.
    pub start_y: f64,
}

/// Popover card preview state when inspecting a node referencing an item.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CardPreviewState {
    /// Node identifier displaying this preview.
    pub node_id: String,
    /// Vault item identifier (ULID).
    pub item_id: String,
    /// Title or prompt summary.
    pub title: String,
    /// Card body or preview content.
    pub content: String,
}

/// Complete in-memory state of the 2D FrogCanvas view.
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasState {
    /// Document holding spatial nodes, edges, and grouping frames.
    pub doc: CanvasDoc,
    /// Relative vault stack path where `_canvas.toml` is stored.
    pub stack_path: String,
    /// Current viewport pan and zoom transformation.
    pub transform: CanvasTransform,
    /// Currently dragged node, if any.
    pub dragging: Option<DraggingState>,
    /// Identifier of the currently selected node.
    pub selected_node: Option<String>,
    /// Identifier of the currently hovered node.
    pub hovered_node: Option<String>,
    /// Active card preview popover, if open.
    pub preview: Option<CardPreviewState>,
    /// Whether unpersisted spatial modifications have occurred.
    pub dirty: bool,
}

impl Default for CanvasState {
    fn default() -> Self {
        new_canvas_state(CanvasDoc::default(), String::new())
    }
}

/// User interactions and events for the canvas view.
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasMsg {
    /// Pan canvas viewport by screen delta pixels `(dx, dy)`.
    Pan(f32, f32),
    /// Multiply zoom factor by the given scale ratio.
    Zoom(f32),
    /// Smooth zoom centered at screen position `(sx, sy)`.
    ZoomAt {
        /// Zoom multiplication factor.
        factor: f32,
        /// Screen pixel center coordinates around which to zoom.
        center: (f32, f32),
    },
    /// Reset canvas viewport to default identity transform.
    ResetView,
    /// Start dragging the specified node.
    DragStart(String),
    /// Update position of currently dragged node by screen delta.
    DragDelta(f32, f32),
    /// Conclude dragging the active node.
    DragEnd,
    /// Select a node or clear selection.
    Select(Option<String>),
    /// Hover over a node or clear hover.
    Hover(Option<String>),
    /// Open the card preview popover for a node.
    OpenPreview(String),
    /// Dismiss the active card preview popover.
    ClosePreview,
    /// Replace the active canvas document.
    SetDoc(CanvasDoc),
}

/// Constructs a default canvas transform with identity scale and zero pan.
pub fn default_canvas_transform() -> CanvasTransform {
    CanvasTransform {
        pan_x: 0.0,
        pan_y: 0.0,
        zoom: 1.0,
    }
}

/// Creates a new canvas state with given document and stack path.
pub fn new_canvas_state(doc: CanvasDoc, stack_path: String) -> CanvasState {
    CanvasState {
        doc,
        stack_path,
        transform: default_canvas_transform(),
        dragging: None,
        selected_node: None,
        hovered_node: None,
        preview: None,
        dirty: false,
    }
}

/// Clamps zoom value into `[MIN_ZOOM, MAX_ZOOM]`, sanitizing NaN or non-finite values.
pub fn clamp_zoom(zoom: f64) -> f64 {
    if !zoom.is_finite() || zoom <= 0.0 {
        1.0
    } else {
        zoom.clamp(MIN_ZOOM, MAX_ZOOM)
    }
}

/// Transforms world coordinate `(wx, wy)` to screen pixel coordinate `(sx, sy)`.
pub fn world_to_screen(tx: &CanvasTransform, wx: f64, wy: f64, center: (f32, f32)) -> (f32, f32) {
    let sx = center.0 + ((wx + tx.pan_x) * tx.zoom) as f32;
    let sy = center.1 + ((wy + tx.pan_y) * tx.zoom) as f32;
    (sx, sy)
}

/// Transforms screen pixel coordinate `(sx, sy)` to world coordinate `(wx, wy)`.
pub fn screen_to_world(tx: &CanvasTransform, sx: f32, sy: f32, center: (f32, f32)) -> (f64, f64) {
    let z = if tx.zoom.abs() < 1e-6 { 1.0 } else { tx.zoom };
    let wx = ((sx - center.0) as f64 / z) - tx.pan_x;
    let wy = ((sy - center.1) as f64 / z) - tx.pan_y;
    (wx, wy)
}
