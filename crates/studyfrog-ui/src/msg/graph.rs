//! Link graph canvas messages.

use studyfrog_store::overview::LinkGraph;

/// The parts of the link graph screen that the rail switches between.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GraphSection {
    /// The drawn graph.
    #[default]
    Map,
    /// The items as a list that can be read and opened.
    List,
}

/// Link graph canvas messages.
#[derive(Debug, Clone, PartialEq)]
pub enum MsgGraph {
    /// The rail selected this part of the screen.
    Section(GraphSection),
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
