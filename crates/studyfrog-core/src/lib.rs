//! Domain types (Stack, Note, Card, Question, Run, ...) as pure data. No I/O, no behaviour blocks; the root of the dependency graph.

pub mod arena;
pub mod civil;
pub mod event;
pub mod idx;
pub mod items;
pub mod library;
pub mod memory;
pub mod meta;
pub mod plan;
pub mod review;
pub mod run;
pub mod scheduler;
pub mod settings;
pub mod stack;
pub mod time;
pub mod uid;

#[cfg(test)]
mod tests;
