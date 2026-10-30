//! Domain types (Stack, Note, Card, Question, Run, ...) as pure data. No I/O, no behaviour blocks; the root of the dependency graph.

pub mod arena;
pub mod idx;
pub mod uid;

#[cfg(test)]
mod tests;
