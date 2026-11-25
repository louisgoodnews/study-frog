//! Checks that log instead of panicking.

/// Returns `condition`. When it is false, logs `message` at Error level for `target`. The caller
/// decides what to do next; this never panics.
pub fn check(target: &str, condition: bool, message: &str) -> bool {
    if !condition {
        studyfrog_log::log_error!(target, "{}", message);
    }
    condition
}
