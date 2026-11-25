//! Debug timers (studyfrog-debug).
use studyfrog_debug::{elapsed_us, timed};

#[test]
fn elapsed_time_grows_and_is_in_microseconds() {
    let timer = timed("debug-timer", "busy wait");
    // Busy-wait for about 1 ms. Thread sleeps are disallowed by clippy.toml.
    while elapsed_us(&timer) < 1_000 {}
    assert!(elapsed_us(&timer) >= 1_000);
}
