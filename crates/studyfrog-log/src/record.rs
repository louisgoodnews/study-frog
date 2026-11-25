//! One log record.

use crate::severity::Level;

/// One record, as kept in the ring and passed to the sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// How severe the record is.
    pub level: Level,
    /// Where the record comes from, for example `ui` or `service`.
    pub target: String,
    /// The message text.
    pub message: String,
    /// Named values, in the order the caller gave them.
    pub fields: Vec<(String, String)>,
    /// Milliseconds since the Unix epoch when the record was made.
    pub unix_ms: u64,
}

/// Milliseconds since the Unix epoch now, or 0 if the clock is before the epoch.
pub(crate) fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}
