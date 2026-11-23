//! The widget commands of the control protocol: the window's widget tree (AccessKit), a widget found
//! by its label, and a click on one. Only a window has a widget tree; a headless instance refuses.

/// What `ui ...` asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiCmd {
    /// The named widgets of the frame, in reading order.
    Tree,
    /// The widgets labelled with this text (exactly, else containing it).
    Find(String),
    /// Click the widget labelled with this text.
    Click(String),
}
