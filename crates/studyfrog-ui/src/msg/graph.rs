//! Link graph canvas messages.

use studyfrog_store::overview::LinkGraph;

/// Link graph canvas messages.
#[derive(Debug, Clone, PartialEq)]
pub enum MsgGraph {
    /// The link graph was read from the index.
    GraphLoaded(Result<LinkGraph, String>),
    /// A frame of the graph canvas of this size (pixels) is being drawn.
    GraphFrame {
        /// Canvas width.
        width: f32,
        /// Canvas height.
        height: f32,
    },
    /// The canvas was dragged by this many pixels.
    GraphPan(f32, f32),
    /// The canvas was scrolled; multiply the zoom by this factor.
    GraphZoom(f32),
    /// The pointer is over this node (or none).
    GraphHover(Option<usize>),
    /// "Re-run layout" was activated.
    GraphRestart,
    /// A node was clicked.
    GraphClick(usize),
}
