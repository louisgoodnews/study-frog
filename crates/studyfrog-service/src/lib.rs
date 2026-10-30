//! Application services: composes the domain crates (store, srs, rehearsal) into use cases. No GUI, no business rules of its own beyond orchestration.

pub mod achievements;
pub mod compare;
pub mod content;
pub mod create;
pub mod day;
pub mod document;
pub mod kanban;
pub mod planning;
pub mod queue;
pub mod queue_data;
pub mod rehearse;
pub mod rerate;
pub mod schedule;
pub mod session;
pub mod settings;
pub mod timezone;
