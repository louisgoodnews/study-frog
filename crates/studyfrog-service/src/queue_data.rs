//! Type aliases for the data the queue builder reads from the store.

use std::collections::HashMap;

use studyfrog_core::uid::Uid;
use studyfrog_store::schedule::StoredMem;

/// Library position of each indexed item.
pub type Positions = HashMap<Uid, u32>;
/// Stored schedule of each scheduled item.
pub type Schedules = HashMap<Uid, StoredMem>;
