//! Moving cards: drag, push, pull and reorder. The WIP rule lives here.

use studyfrog_core::uid::Uid;

use crate::model::{Board, BoardError, Card, MoveMode, find_card, find_column};

/// Remove a card, returning it and the board without it.
fn take(b: &Board, id: Uid) -> Result<(Card, Board, usize), BoardError> {
    let (c, i) = find_card(b, id).ok_or(BoardError::NoSuchCard(id))?;
    let mut next = b.clone();
    let card = next.columns[c].cards.remove(i);
    Ok((card, next, c))
}

/// Move a card to column `to` at `index` (clamped to the end). Reordering inside a column is always
/// allowed. Entering another column is refused in `Strict` mode when it is full.
pub fn move_card(
    b: &Board,
    id: Uid,
    to: Uid,
    index: usize,
    mode: MoveMode,
) -> Result<Board, BoardError> {
    let target = find_column(b, to).ok_or(BoardError::NoSuchColumn(to))?;
    let (card, mut next, from) = take(b, id)?;
    let col = &mut next.columns[target];
    if from != target
        && mode == MoveMode::Strict
        && let Some(limit) = col.wip_limit.filter(|l| col.cards.len() >= *l as usize)
    {
        return Err(BoardError::WipLimit { column: to, limit });
    }
    col.cards.insert(index.min(col.cards.len()), card);
    Ok(next)
}

/// Push a card to the top of the next column.
pub fn push(b: &Board, id: Uid, mode: MoveMode) -> Result<Board, BoardError> {
    let (c, _) = find_card(b, id).ok_or(BoardError::NoSuchCard(id))?;
    let next = b.columns.get(c + 1).ok_or(BoardError::AtLastColumn)?;
    move_card(b, id, next.id, 0, mode)
}

/// Pull a card back to the top of the previous column.
pub fn pull(b: &Board, id: Uid, mode: MoveMode) -> Result<Board, BoardError> {
    let (c, _) = find_card(b, id).ok_or(BoardError::NoSuchCard(id))?;
    let prev = c
        .checked_sub(1)
        .and_then(|p| b.columns.get(p))
        .ok_or(BoardError::AtFirstColumn)?;
    move_card(b, id, prev.id, 0, mode)
}

/// Move a card up (`delta` < 0) or down (`delta` > 0) inside its column, clamped to the ends.
pub fn shift(b: &Board, id: Uid, delta: i32) -> Result<Board, BoardError> {
    let (c, i) = find_card(b, id).ok_or(BoardError::NoSuchCard(id))?;
    let len = b.columns[c].cards.len() as i64;
    let to = (i as i64 + i64::from(delta)).clamp(0, len - 1) as usize;
    move_card(b, id, b.columns[c].id, to, MoveMode::Strict)
}
