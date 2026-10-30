//! The board as plain data. Every operation returns a new board.

use studyfrog_core::uid::Uid;

/// A task card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// Identity.
    pub id: Uid,
    /// Title.
    pub title: String,
    /// Free-form notes.
    pub notes: String,
    /// A stack this task is about, if any.
    pub stack: Option<Uid>,
    /// Story points, if estimated.
    pub points: Option<u32>,
}

/// A column and the cards in it, top first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    /// Identity.
    pub id: Uid,
    /// Name.
    pub name: String,
    /// Work-in-progress limit; `None` means unlimited.
    pub wip_limit: Option<u32>,
    /// Cards in order.
    pub cards: Vec<Card>,
}

/// A kanban board: columns from left to right.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Board {
    /// Columns in order.
    pub columns: Vec<Column>,
}

/// How strictly WIP limits are applied when cards enter a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveMode {
    /// Refuse a move that would exceed the limit.
    Strict,
    /// Allow it; the column is then over its limit (visible, but the team chose it).
    Force,
}

/// Why an operation was refused. The board is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardError {
    /// No card has this id.
    NoSuchCard(Uid),
    /// No column has this id.
    NoSuchColumn(Uid),
    /// The target column is full.
    WipLimit {
        /// The full column.
        column: Uid,
        /// Its limit.
        limit: u32,
    },
    /// The card is already in the last column.
    AtLastColumn,
    /// The card is already in the first column.
    AtFirstColumn,
    /// An id is already in use.
    DuplicateId(Uid),
    /// The only column cannot be removed.
    LastRemainingColumn,
}

/// Where a card is: (column index, index in the column).
pub type Place = (usize, usize);

/// Locate a card.
pub fn find_card(b: &Board, id: Uid) -> Option<Place> {
    b.columns
        .iter()
        .enumerate()
        .find_map(|(c, col)| col.cards.iter().position(|k| k.id == id).map(|i| (c, i)))
}

/// Index of a column.
pub fn find_column(b: &Board, id: Uid) -> Option<usize> {
    b.columns.iter().position(|c| c.id == id)
}

/// Total number of cards.
pub fn card_count(b: &Board) -> usize {
    b.columns.iter().map(|c| c.cards.len()).sum()
}

/// True when the column holds more cards than its limit.
pub fn is_over_limit(c: &Column) -> bool {
    c.wip_limit.is_some_and(|l| c.cards.len() > l as usize)
}

/// Ids of the columns that are over their limit.
pub fn over_limit(b: &Board) -> Vec<Uid> {
    b.columns
        .iter()
        .filter(|c| is_over_limit(c))
        .map(|c| c.id)
        .collect()
}

/// A new card with no notes, stack or estimate.
pub fn card_new(id: Uid, title: &str) -> Card {
    Card {
        id,
        title: title.to_owned(),
        notes: String::new(),
        stack: None,
        points: None,
    }
}

/// The default board: Backlog, Doing (limit 3), Review, Done. `ids` names the four columns.
pub fn board_default(ids: [Uid; 4]) -> Board {
    let column = |id, name: &str, wip_limit| Column {
        id,
        name: name.to_owned(),
        wip_limit,
        cards: Vec::new(),
    };
    Board {
        columns: vec![
            column(ids[0], "Backlog", None),
            column(ids[1], "Doing", Some(3)),
            column(ids[2], "Review", None),
            column(ids[3], "Done", None),
        ],
    }
}

/// A refusal in words for the user.
pub fn describe_error(b: &Board, e: &BoardError) -> String {
    let name = |id: Uid| {
        find_column(b, id)
            .map_or("that column", |i| b.columns[i].name.as_str())
            .to_owned()
    };
    match e {
        BoardError::WipLimit { column, limit } => {
            format!(
                "\"{}\" is full: its work-in-progress limit is {limit}.",
                name(*column)
            )
        }
        BoardError::NoSuchCard(_) => "That card no longer exists.".to_owned(),
        BoardError::NoSuchColumn(_) => "That column no longer exists.".to_owned(),
        BoardError::AtLastColumn => "The card is already in the last column.".to_owned(),
        BoardError::AtFirstColumn => "The card is already in the first column.".to_owned(),
        BoardError::DuplicateId(_) => "That id is already in use.".to_owned(),
        BoardError::LastRemainingColumn => "The last column cannot be removed.".to_owned(),
    }
}
