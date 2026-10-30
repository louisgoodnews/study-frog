//! Type aliases for the data the queue builder reads from the store.

use std::collections::HashMap;

use studyfrog_core::uid::Uid;
use studyfrog_store::schedule::Scheduled;

/// Library position of each indexed item.
pub type Positions = HashMap<Uid, u32>;
/// Schedule of each scheduled item, whatever algorithm scheduled it.
pub type Schedules = HashMap<Uid, Scheduled>;
