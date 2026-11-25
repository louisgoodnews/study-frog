//! Process-wide state: the minimum level and the ring of recent records.
//!
//! The level is one atomic, so checking it costs a single load. The ring is a mutex-guarded deque
//! that keeps at most `RING_CAPACITY` records.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::record::Record;
use crate::severity::{Level, from_u8};

/// How many recent records the ring keeps.
pub const RING_CAPACITY: usize = 512;

static LEVEL: AtomicU8 = AtomicU8::new(Level::Info as u8);
static RING: Mutex<VecDeque<Record>> = Mutex::new(VecDeque::new());

/// Sets the minimum level. Records above it are dropped. Call once at start-up.
pub fn set_level(level: Level) {
    LEVEL.store(level as u8, Ordering::Relaxed);
}

/// The minimum level now in force. The default is `Info`.
pub fn level() -> Level {
    from_u8(LEVEL.load(Ordering::Relaxed))
}

/// The most recent `n` records, oldest first. Fewer are returned while the ring is not yet full.
pub fn recent(n: usize) -> Vec<Record> {
    let ring = lock(&RING);
    let skip = ring.len().saturating_sub(n);
    ring.iter().skip(skip).cloned().collect()
}

/// Keeps `record` in the ring, dropping the oldest record when the ring is full.
pub(crate) fn remember(record: Record) {
    let mut ring = lock(&RING);
    if ring.len() >= RING_CAPACITY {
        ring.pop_front();
    }
    ring.push_back(record);
}

/// Locks a mutex. A poisoned lock is used anyway: a log must keep working after a panic elsewhere.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Read-locks an `RwLock`, ignoring poisoning as `lock` does.
pub(crate) fn read_lock<T>(rwlock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    rwlock.read().unwrap_or_else(PoisonError::into_inner)
}

/// Write-locks an `RwLock`, ignoring poisoning as `lock` does.
pub(crate) fn write_lock<T>(rwlock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    rwlock.write().unwrap_or_else(PoisonError::into_inner)
}
