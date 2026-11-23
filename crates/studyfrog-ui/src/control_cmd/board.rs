//! The board commands of the control protocol: the columns with their cards, one column, and a new
//! card in a column. A card's status is the column it is in.

/// What `board ...` asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardCmd {
    /// Open the board and answer with every column and its cards.
    List,
    /// Open the board and answer with one column (by name) and its cards.
    Column(String),
    /// Add a card with this title to the column with this name (`None`: the first column).
    Add {
        /// The title of the card.
        title: String,
        /// The column the card goes to.
        column: Option<String>,
    },
}
