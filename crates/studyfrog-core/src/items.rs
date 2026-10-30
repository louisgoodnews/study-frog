//! Item kinds that live inside a stack: Note, Card, Question.
//!
//! Each kind has its own arena in [`Library`](crate::library::Library); an item refers to its
//! stack by [`Idx<Stack>`].

use crate::idx::Idx;
use crate::meta::Meta;
use crate::stack::Stack;

/// Free-form Markdown document.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// Common fields.
    pub meta: Meta,
    /// Owning stack.
    pub stack: Idx<Stack>,
    /// The `rehearse_title` key (default false).
    pub rehearse_title: bool,
    /// Markdown body without front-matter.
    pub body: String,
}

/// Flashcard: text before the `???` line is the front, text after is the back.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    /// Common fields.
    pub meta: Meta,
    /// Owning stack.
    pub stack: Idx<Stack>,
    /// The `reversible` key (default false).
    pub reversible: bool,
    /// Prompt side.
    pub front: String,
    /// Answer side.
    pub back: String,
}

/// Prompt with a reference answer, graded by the SLM.
#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    /// Common fields.
    pub meta: Meta,
    /// Owning stack.
    pub stack: Idx<Stack>,
    /// Prompt text (before `???`).
    pub prompt: String,
    /// Reference answer (after `???`).
    pub reference: String,
    /// The `accepted` key: alternative reference answers.
    pub accepted: Vec<String>,
    /// The `rubric` key: required key points.
    pub rubric: Vec<String>,
    /// The `min_score` key, resolved to [`DEFAULT_MIN_SCORE`] when absent.
    pub min_score: f32,
}

/// Default `min_score` from the markdown schema.
pub const DEFAULT_MIN_SCORE: f32 = 0.7;
