//! The one entry point that every macro and caller goes through.

use crate::record::{Record, now_unix_ms};
use crate::severity::Level;
use crate::sink;
use crate::state;

/// Emits one record if `level` is enabled. Records above the minimum level are dropped before
/// anything is copied. Otherwise the record goes into the ring and then to the sink.
/// Field values are already text; the macros format them with `Display`.
pub fn emit(level: Level, target: &str, message: &str, fields: &[(&str, &str)]) {
    if level > state::level() {
        return;
    }
    let record = Record {
        level,
        target: target.to_string(),
        message: message.to_string(),
        fields: fields
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect(),
        unix_ms: now_unix_ms(),
    };
    sink::deliver(&record);
    state::remember(record);
}
