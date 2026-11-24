//! Import and picture messages.

use crate::import_ui::ImportOutcome;

/// Import and picture messages.
#[derive(Debug, Clone, PartialEq)]
// The largest payload is a rare save or import outcome; boxing it would change every call site.
#[allow(clippy::large_enum_variant)]
pub enum MsgImport {
    /// The path of the file to import changed.
    ImportPathEdited(String),
    /// "Browse" was activated next to the import path.
    BrowseImportClicked,
    /// The file dialog closed: the chosen file, or `None`.
    ImportFilePicked(Option<String>),
    /// "Import cards" was activated.
    ImportClicked,
    /// Cards were made by an import or a card type (or not).
    Imported(Result<ImportOutcome, String>),
    /// The user asked to add a picture to a text area.
    PickImage(crate::image_ui::ImageTarget),
    /// A picture was copied into the vault (or the user cancelled).
    ImageAdded(crate::image_ui::ImageTarget, crate::image_ui::Added),
}
