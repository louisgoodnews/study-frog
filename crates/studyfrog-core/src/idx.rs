//! Typed arena index.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Handle to an element of an [`Arena<T>`](crate::arena::Arena). In-memory only: never persist
/// it, use [`Uid`](crate::uid::Uid) for that.
pub struct Idx<T> {
    /// Position in the arena.
    pub raw: u32,
    /// Ties the index to its element type without owning a `T`.
    pub marker: PhantomData<fn() -> T>,
}

/// Build an index from a raw position.
pub fn idx<T>(raw: u32) -> Idx<T> {
    Idx {
        raw,
        marker: PhantomData,
    }
}

// Manual impls: `derive` would require `T: Clone`, `T: Eq`, ... which an index must not need.

impl<T> Clone for Idx<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Idx<T> {}

impl<T> PartialEq for Idx<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T> Eq for Idx<T> {}

impl<T> PartialOrd for Idx<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Idx<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T> Hash for Idx<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T> fmt::Debug for Idx<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Idx({})", self.raw)
    }
}
