//! Debugging helpers on top of studyfrog-log: timers, checkpoints, value dumps, checks and counters.
//! Everything logs through studyfrog-log, so the level and the sink set there apply here too.
#![forbid(unsafe_code)]

mod check;
mod counts;
mod dump;
mod macros;
mod timer;

pub use check::check;
pub use counts::{count, counts};
pub use dump::{dump, truncate_chars};
pub use timer::{Timer, checkpoint, elapsed_us, timed};
