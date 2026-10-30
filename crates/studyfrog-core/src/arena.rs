//! Append-only arena addressed by [`Idx`]. Data plus free functions; no hidden state.

use crate::idx::{Idx, idx};

/// Contiguous storage for `T`. Elements are only appended, so an [`Idx`] stays valid.
#[derive(Debug, Clone)]
pub struct Arena<T> {
    /// Elements in insertion order. Use [`arena_push`] to append so indices stay typed.
    pub items: Vec<T>,
}

/// The arena already holds `u32::MAX` elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArenaFull;

/// An empty arena.
pub fn arena_new<T>() -> Arena<T> {
    Arena { items: Vec::new() }
}

/// An empty arena with room for `capacity` elements.
pub fn arena_with_capacity<T>(capacity: usize) -> Arena<T> {
    Arena {
        items: Vec::with_capacity(capacity),
    }
}

/// Append `value` and return its index.
pub fn arena_push<T>(arena: &mut Arena<T>, value: T) -> Result<Idx<T>, ArenaFull> {
    let raw = u32::try_from(arena.items.len()).map_err(|_| ArenaFull)?;
    arena.items.push(value);
    Ok(idx(raw))
}

/// Borrow the element at `at`, if present.
pub fn arena_get<T>(arena: &Arena<T>, at: Idx<T>) -> Option<&T> {
    arena.items.get(at.raw as usize)
}

/// Mutably borrow the element at `at`, if present.
pub fn arena_get_mut<T>(arena: &mut Arena<T>, at: Idx<T>) -> Option<&mut T> {
    arena.items.get_mut(at.raw as usize)
}

/// Number of elements.
pub fn arena_len<T>(arena: &Arena<T>) -> usize {
    arena.items.len()
}

/// Iterate `(index, element)` pairs in insertion order without allocating.
pub fn arena_iter<T>(arena: &Arena<T>) -> impl Iterator<Item = (Idx<T>, &T)> {
    arena
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, v)| u32::try_from(i).ok().map(|raw| (idx(raw), v)))
}
