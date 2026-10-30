//! Domain types (Stack, Note, Card, Question, Run, ...) as pure data. No I/O, no behaviour blocks; the root of the dependency graph.

pub mod arena;
pub mod idx;
pub mod items;
pub mod library;
pub mod meta;
pub mod review;
pub mod stack;
pub mod time;
pub mod uid;

#[cfg(test)]
mod tests;
