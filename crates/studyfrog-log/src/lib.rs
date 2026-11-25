//! Logging for StudyFrog, written in-house with no external crates.
//!
//! The minimum level and the sink are process-wide and are set once at start-up. Records below the
//! minimum are dropped before they are formatted. Every kept record goes to the sink (stderr by
//! default) and into a ring of the most recent records, which `recent` returns for debugging.
#![forbid(unsafe_code)]

mod dispatch;
mod file;
mod format;
mod macros;
mod record;
mod severity;
mod sink;
mod state;

pub use dispatch::emit;
pub use file::file_sink;
pub use format::format_line;
pub use record::Record;
pub use severity::{Level, name, parse_level};
pub use sink::{Sink, set_sink};
pub use state::{RING_CAPACITY, level, recent, set_level};
