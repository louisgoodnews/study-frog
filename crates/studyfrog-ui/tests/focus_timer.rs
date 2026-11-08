#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The focus timer's phases on an injected clock (STORY-0112).

use proptest::prelude::*;
use studyfrog_ui::focus_timer::{
    PRESETS, Phase, Preset, clock, elapsed, focus_new, is_idle, pause, remaining, reset,
    set_preset, skip_break, start, tick,
};

const WALL: i64 = 1_794_096_000_000;

#[test]
fn a_work_block_ends_on_time_and_becomes_a_waiting_break() {
    let mut f = focus_new();
    assert!(is_idle(&f));
    start(&mut f, 10.0, WALL);
    assert_eq!(remaining(&f, 10.0), 25 * 60);
    assert_eq!(clock(remaining(&f, 70.5)), "24:00");
    assert!(tick(&mut f, 10.0 + 1499.0, WALL).is_none());
    let log = tick(&mut f, 10.0 + 1500.0, WALL + 1_500_000).unwrap();
    assert_eq!(
        (log.minutes, log.started_ms, log.ended_ms),
        (25, WALL, WALL + 1_500_000)
    );
    assert_eq!(f.phase, Phase::ShortBreak);
    assert_eq!(f.blocks, 1);
    assert!(f.done.is_some(), "the card waits for the user");
    assert!(
        tick(&mut f, 9999.0, WALL).is_none(),
        "the break waits to be started"
    );
}

#[test]
fn pauses_do_not_count_and_every_fourth_break_is_long() {
    let mut f = focus_new();
    start(&mut f, 0.0, WALL);
    pause(&mut f, 600.0);
    assert_eq!(elapsed(&f, 5000.0), 600.0, "paused time is not focus");
    start(&mut f, 5000.0, WALL);
    let log = tick(&mut f, 5900.0, WALL).unwrap();
    assert_eq!(log.minutes, 25);
    for n in 2..=4 {
        skip_break(&mut f, 0.0, WALL);
        assert_eq!(f.phase, Phase::Work);
        tick(&mut f, 1500.0, WALL).unwrap();
        let want = if n == 4 {
            Phase::LongBreak
        } else {
            Phase::ShortBreak
        };
        assert_eq!(f.phase, want, "after block {n}");
    }
    start(&mut f, 0.0, WALL);
    assert!(tick(&mut f, 15.0 * 60.0, WALL).is_none());
    assert_eq!(f.phase, Phase::Work, "the long break ended");
}

#[test]
fn presets_and_custom_lengths_are_kept_within_bounds() {
    let mut f = focus_new();
    set_preset(&mut f, PRESETS[1]);
    assert_eq!(
        f.preset,
        Preset {
            work: 50,
            short: 10,
            long: 20
        }
    );
    set_preset(
        &mut f,
        Preset {
            work: 0,
            short: 500,
            long: 0,
        },
    );
    assert_eq!(
        f.preset,
        Preset {
            work: 1,
            short: 120,
            long: 1
        }
    );
    start(&mut f, 0.0, WALL);
    reset(&mut f);
    assert!(is_idle(&f));
}

proptest! {
    #[test]
    fn elapsed_time_only_grows_while_running(steps in prop::collection::vec((0.0_f64..400.0, any::<bool>()), 1..30)) {
        let mut f = focus_new();
        set_preset(&mut f, Preset { work: 600, short: 5, long: 15 });
        let (mut now, mut counted) = (0.0, 0.0);
        for (dt, run) in steps {
            if run { start(&mut f, now, WALL) } else { pause(&mut f, now) }
            now += dt;
            if run { counted += dt; }
            pause(&mut f, now);
            prop_assert!((elapsed(&f, now + 1000.0) - counted).abs() < 1e-6);
        }
    }
}
