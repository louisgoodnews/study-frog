//! Adding, changing and removing cards and columns.

use studyfrog_core::uid::Uid;

use crate::model::{Board, BoardError, Card, Column, MoveMode, find_card, find_column};

fn id_in_use(b: &Board, id: Uid) -> bool {
    b.columns
        .iter()
        .any(|c| c.id == id || c.cards.iter().any(|k| k.id == id))
}

/// Add a card to the end of a column. `Strict` refuses a full column.
pub fn add_card(b: &Board, column: Uid, card: Card, mode: MoveMode) -> Result<Board, BoardError> {
    let at = find_column(b, column).ok_or(BoardError::NoSuchColumn(column))?;
    if id_in_use(b, card.id) {
        return Err(BoardError::DuplicateId(card.id));
    }
    let mut next = b.clone();
    let col = &mut next.columns[at];
    if let (MoveMode::Strict, Some(limit)) = (mode, col.wip_limit)
        && col.cards.len() >= limit as usize
    {
        return Err(BoardError::WipLimit { column, limit });
    }
    col.cards.push(card);
    Ok(next)
}

/// Replace a card's title, notes, stack link and estimate (its place is kept).
pub fn update_card(b: &Board, id: Uid, f: impl FnOnce(&mut Card)) -> Result<Board, BoardError> {
    let (c, i) = find_card(b, id).ok_or(BoardError::NoSuchCard(id))?;
    let mut next = b.clone();
    f(&mut next.columns[c].cards[i]);
    next.columns[c].cards[i].id = id;
    Ok(next)
}

/// Delete a card.
pub fn remove_card(b: &Board, id: Uid) -> Result<Board, BoardError> {
    let (c, i) = find_card(b, id).ok_or(BoardError::NoSuchCard(id))?;
    let mut next = b.clone();
    next.columns[c].cards.remove(i);
    Ok(next)
}

/// Insert a new empty column at `index` (clamped).
pub fn add_column(
    b: &Board,
    id: Uid,
    name: &str,
    wip_limit: Option<u32>,
    index: usize,
) -> Result<Board, BoardError> {
    if id_in_use(b, id) {
        return Err(BoardError::DuplicateId(id));
    }
    let mut next = b.clone();
    let at = index.min(next.columns.len());
    next.columns.insert(
        at,
        Column {
            id,
            name: name.to_owned(),
            wip_limit,
            cards: Vec::new(),
        },
    );
    Ok(next)
}

/// Rename a column and/or change its limit. Lowering a limit below the card count is allowed: the
/// column is then over its limit and nothing moves by itself.
pub fn edit_column(
    b: &Board,
    id: Uid,
    name: &str,
    wip_limit: Option<u32>,
) -> Result<Board, BoardError> {
    let at = find_column(b, id).ok_or(BoardError::NoSuchColumn(id))?;
    let mut next = b.clone();
    next.columns[at].name = name.to_owned();
    next.columns[at].wip_limit = wip_limit;
    Ok(next)
}

/// Remove a column. Its cards go to the end of the previous column (the next one when it was first).
/// That may leave the receiving column over its limit. The last column cannot be removed.
pub fn remove_column(b: &Board, id: Uid) -> Result<Board, BoardError> {
    let at = find_column(b, id).ok_or(BoardError::NoSuchColumn(id))?;
    if b.columns.len() == 1 {
        return Err(BoardError::LastRemainingColumn);
    }
    let mut next = b.clone();
    let gone = next.columns.remove(at);
    let receiver = at.saturating_sub(1);
    next.columns[receiver].cards.extend(gone.cards);
    Ok(next)
}

/// Move a column to `index` (clamped).
pub fn move_column(b: &Board, id: Uid, index: usize) -> Result<Board, BoardError> {
    let at = find_column(b, id).ok_or(BoardError::NoSuchColumn(id))?;
    let mut next = b.clone();
    let col = next.columns.remove(at);
    next.columns.insert(index.min(next.columns.len()), col);
    Ok(next)
}
