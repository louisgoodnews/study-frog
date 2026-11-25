//! Timers and checkpoints.

use std::time::Instant;

use studyfrog_log::Level;

/// A timer that logs its elapsed time at Debug level when it is dropped, with the field
/// `elapsed_us`. Bind it to a named variable (`let _timer = ...`); `let _ = ...` drops it at once.
pub struct Timer {
    target: String,
    name: String,
    start: Instant,
}

/// Starts a timer called `name` for `target`. It logs when it goes out of scope.
pub fn timed(target: &str, name: &str) -> Timer {
    Timer {
        target: target.to_string(),
        name: name.to_string(),
        start: Instant::now(),
    }
}

/// Microseconds since the timer started, saturating at `u64::MAX`.
pub fn elapsed_us(timer: &Timer) -> u64 {
    u64::try_from(timer.start.elapsed().as_micros()).unwrap_or(u64::MAX)
}

/// Logs a checkpoint named `name` for `target` at Trace level.
pub fn checkpoint(target: &str, name: &str) {
    studyfrog_log::log_trace!(target, "{}", name);
}

impl Drop for Timer {
    fn drop(&mut self) {
        // Skip the formatting when Debug records are off.
        if Level::Debug <= studyfrog_log::level() {
            let micros = elapsed_us(self).to_string();
            studyfrog_log::emit(
                Level::Debug,
                &self.target,
                &self.name,
                &[("elapsed_us", micros.as_str())],
            );
        }
    }
}
