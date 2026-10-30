//! The clock seam: the engine takes timestamps as input, the shell obtains them from a [`Clock`].

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use crate::input::{Action, Input};

/// Source of monotonic milliseconds since the run began. The one place a trait is warranted here:
/// production uses the system clock, tests use a manual one.
pub trait Clock {
    /// Milliseconds since the clock started; never decreases.
    fn now_ms(&self) -> u32;
}

/// Monotonic system clock started at construction.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    /// Moment the clock started.
    pub started: Instant,
}

/// Clock whose time is set by hand.
#[derive(Debug, Default)]
pub struct ManualClock {
    /// Current time in milliseconds.
    pub now: AtomicU32,
}

/// Start a system clock now.
pub fn system_clock() -> SystemClock {
    SystemClock {
        started: Instant::now(),
    }
}

fn elapsed_ms(started: &Instant) -> u32 {
    u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX)
}

/// Move a manual clock forward by `ms`.
pub fn advance(clock: &ManualClock, ms: u32) {
    clock.now.fetch_add(ms, Ordering::Relaxed);
}

impl Clock for SystemClock {
    fn now_ms(&self) -> u32 {
        elapsed_ms(&self.started)
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u32 {
        self.now.load(Ordering::Relaxed)
    }
}

/// Stamp an action with the clock's current time.
pub fn input_now(clock: &impl Clock, action: Action) -> Input {
    Input {
        t_ms: clock.now_ms(),
        action,
    }
}
