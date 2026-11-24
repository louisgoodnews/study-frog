//! Document editor messages.

pub use crate::edit_form::EditAction;
use studyfrog_core::uid::Uid;
use studyfrog_service::document::{LoadedDoc, Saved};

/// Document editor messages.
#[derive(Debug, Clone, PartialEq)]
pub enum MsgEditor {
    /// A document was read for editing.
    DocumentLoaded(Uid, Result<LoadedDoc, String>),
    /// A field of the editor changed.
    Edit(EditAction),
    /// Save now (the editor saves by itself; this is for tests and the keyboard).
    SaveClicked,
    /// "Keep mine" was activated after the file changed outside StudyFrog.
    KeepMineClicked,
    /// The document was saved (or not).
    Saved(Result<Saved, String>),
    /// "Revert" was activated.
    RevertClicked,
    /// "Reload from disk" was activated.
    ReloadClicked,
    /// "Back" was activated in the editor.
    CloseEditor,
    /// A wikilink in the preview was clicked.
    OpenLink(String),
    /// The item a wikilink points to (`None`: no item has that title).
    LinkResolved(String, Result<Option<Uid>, String>),
}
