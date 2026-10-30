//! The in-memory library: one arena per kind (structure of arrays), no `Item` enum.

use crate::arena::{Arena, arena_new};
use crate::items::{Card, Note, Question};
use crate::stack::Stack;

/// All loaded content. Kinds are stored separately so scans over one kind (for example due cards)
/// touch contiguous memory and never branch on a tag.
#[derive(Debug, Clone)]
pub struct Library {
    /// Stack definitions.
    pub stacks: Arena<Stack>,
    /// Notes.
    pub notes: Arena<Note>,
    /// Cards.
    pub cards: Arena<Card>,
    /// Questions.
    pub questions: Arena<Question>,
}

/// An empty library.
pub fn library_new() -> Library {
    Library {
        stacks: arena_new(),
        notes: arena_new(),
        cards: arena_new(),
        questions: arena_new(),
    }
}
