//! The records an extension provides.

/// A card an importer found, before it is written to the vault.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportedCard {
    /// The prompt side.
    pub front: String,
    /// The answer side.
    pub back: String,
    /// A title; empty means "derive it from the front".
    pub title: String,
    /// Tags to put on the card.
    pub tags: Vec<String>,
}

/// The two faces of a card a card type produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Faces {
    /// The prompt side.
    pub front: String,
    /// The answer side.
    pub back: String,
}

/// Reads the text of a file into cards. Must not panic and must not do I/O: the application reads
/// the file and hands over its text.
#[derive(Debug, Clone, Copy)]
pub struct Importer {
    /// A stable identifier, lowercase words joined by `-`.
    pub id: &'static str,
    /// The name shown to the user.
    pub name: &'static str,
    /// File extensions it can read, lowercase and without the dot.
    pub extensions: &'static [&'static str],
    /// The conversion; an error says what is wrong with the input, with a line number when it can.
    pub import: fn(&str) -> Result<Vec<ImportedCard>, String>,
}

/// Turns one source text into cards. Must not panic and must not do I/O.
#[derive(Debug, Clone, Copy)]
pub struct CardType {
    /// A stable identifier, lowercase words joined by `-`.
    pub id: &'static str,
    /// The name shown to the user.
    pub name: &'static str,
    /// One line telling the user what to type.
    pub hint: &'static str,
    /// The conversion; an error says what is wrong with the source.
    pub expand: fn(&str) -> Result<Vec<Faces>, String>,
}

/// Everything extensions have registered.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    /// Importers, in registration order.
    pub importers: Vec<Importer>,
    /// Card types, in registration order.
    pub card_types: Vec<CardType>,
}
