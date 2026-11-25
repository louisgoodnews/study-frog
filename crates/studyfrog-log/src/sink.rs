//! Where records go: stderr by default, or the sink the caller sets once at start-up.

use std::io::Write;
use std::sync::RwLock;

use crate::format::format_line;
use crate::record::Record;
use crate::state::{read_lock, write_lock};

/// A destination for records. It is called for every record that passes the minimum level.
pub type Sink = Box<dyn Fn(&Record) + Send + Sync>;

static SINK: RwLock<Option<Sink>> = RwLock::new(None);

/// Replaces the sink. Until one is set, each record is written to stderr as one line.
pub fn set_sink(sink: Sink) {
    *write_lock(&SINK) = Some(sink);
}

/// Hands a record to the current sink, or to stderr when none is set.
pub(crate) fn deliver(record: &Record) {
    match &*read_lock(&SINK) {
        Some(sink) => sink(record),
        None => write_stderr(record),
    }
}

fn write_stderr(record: &Record) {
    // A failed write to stderr has nowhere else to be reported, so the error is dropped.
    let _ = writeln!(std::io::stderr().lock(), "{}", format_line(record));
}
