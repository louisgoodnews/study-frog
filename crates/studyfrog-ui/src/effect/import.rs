//! Import, card creation and picture effects.

/// Import, card creation and picture effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectImport {
    /// Ask the user for a file to import cards from.
    PickImportFile,
    /// Read a file and import its cards into a stack with the importer for its extension.
    ImportFile {
        /// Vault-relative stack folder.
        folder: String,
        /// Absolute path of the file.
        path: String,
    },
    /// Make cards from one source text with a registered card type.
    CreateFromType {
        /// Vault-relative stack folder.
        folder: String,
        /// The card type's id.
        type_id: String,
        /// The text the user wrote.
        source: String,
    },
    /// Create a stack, note, card or question.
    CreateItem {
        /// What to create.
        kind: studyfrog_service::create_more::NewKind,
        /// Title (empty: from the text).
        title: String,
        /// Description, body, front or prompt.
        text: String,
        /// Back or reference answer.
        answer: String,
        /// Tags.
        tags: Vec<String>,
        /// The stack it goes into.
        parent: Option<String>,
        /// Another stack that references the item (optional).
        also_in: Option<String>,
        /// For a stack: due date and reminders.
        due: Option<studyfrog_service::track::DueDate>,
        /// Priority and difficulty chosen.
        choices: crate::modal_types::Choices,
    },
    /// Choose a picture, copy it into the vault and report where it went.
    AddImage(crate::image_ui::ImageTarget),
    /// Create a card in the stack at `folder`.
    #[rustfmt::skip]
    CreateCard { #[doc = "Folder."] folder: String, #[doc = "Title."] title: String, #[doc = "Front."] front: String, #[doc = "Back."] back: String },
}
