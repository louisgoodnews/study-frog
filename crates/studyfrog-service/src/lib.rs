//! Application services: composes the domain crates (store, srs, rehearsal) into use cases. No GUI, no business rules of its own beyond orchestration.

pub mod queue;
pub mod queue_data;
pub mod rehearse;
pub mod schedule;
