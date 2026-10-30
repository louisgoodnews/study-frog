//! Timestamps as plain data.

/// Milliseconds since the Unix epoch (UTC). `core` carries instants as integers; conversion to
/// calendar types (`chrono`/`time`) happens at the edges (md, calendar, ui).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixMs(pub i64);
