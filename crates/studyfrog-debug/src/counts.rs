//! Named counters: how many times something happened during the session.

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

static COUNTS: OnceLock<Mutex<BTreeMap<String, u64>>> = OnceLock::new();

fn registry() -> MutexGuard<'static, BTreeMap<String, u64>> {
    COUNTS
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Adds one to the counter called `name`, creating it at one if it does not exist yet.
pub fn count(name: &str) {
    let mut map = registry();
    match map.get_mut(name) {
        Some(value) => *value = value.saturating_add(1),
        None => {
            map.insert(name.to_string(), 1);
        }
    }
}

/// Every counter and its value, sorted by name.
pub fn counts() -> Vec<(String, u64)> {
    registry()
        .iter()
        .map(|(name, value)| (name.clone(), *value))
        .collect()
}
