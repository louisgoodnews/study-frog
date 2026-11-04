//! Type aliases for the data the queue builder reads from the store.

use std::collections::HashMap;

use studyfrog_core::uid::Uid;
use studyfrog_store::schedule::Scheduled;

/// Schedule of each scheduled item, whatever algorithm scheduled it.
pub type Schedules = HashMap<Uid, Scheduled>;

/// Schedule of each (item, stack) pair, whatever algorithm scheduled it.
pub type StackSchedules = HashMap<(Uid, Uid), Scheduled>;
