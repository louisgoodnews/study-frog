//! Local small-language-model boundary: the `SlmRuntime` trait, the pure scoring functions over
//! embeddings, and an in-house lexical runtime that works offline with no model download.
//!
//! Scoring never touches a runtime's internals: swapping the runtime changes the numbers, not the
//! logic (ADR 0004, [scoring spec](../../../docs/specs/scoring.md)).

pub mod lexical;
pub mod models;
pub mod rating;
pub mod runtime;
pub mod score;
pub mod text;
pub mod vector;
