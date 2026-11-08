#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Within-session fatigue and the run log it is read from (STORY-0103).

use proptest::prelude::*;
use studyfrog_core::event::{EventKind, RunEvent};
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_report::fatigue::{fatigue_report, latency_spike, pace_rate};
use studyfrog_report::session_log::{SessionSample, session_samples};

fn ev(run: u128, seq: u32, t_ms: u32, item: Option<u128>, kind: EventKind) -> RunEvent {
    RunEvent {
        run: Uid(run),
        seq,
        t_ms,
        item: item.map(Uid),
        kind,
    }
}

#[test]
fn the_log_gives_position_active_time_and_latency_without_pauses() {
    let events = [
        ev(1, 0, 0, None, EventKind::RunStarted),
        ev(1, 1, 0, Some(10), EventKind::ItemShown),
        ev(1, 2, 3_000, Some(10), EventKind::ItemFlipped),
        ev(1, 3, 4_000, Some(10), EventKind::ItemRated(Rating::Good)),
        ev(1, 4, 5_000, None, EventKind::RunPaused),
        ev(1, 5, 65_000, None, EventKind::RunResumed),
        ev(1, 6, 66_000, Some(11), EventKind::ItemShown),
        ev(1, 7, 67_000, None, EventKind::RunPaused),
        ev(1, 8, 77_000, None, EventKind::RunResumed),
        ev(1, 9, 80_000, Some(11), EventKind::ItemFlipped),
        ev(1, 10, 81_000, Some(11), EventKind::ItemRated(Rating::Again)),
        ev(1, 11, 82_000, Some(12), EventKind::ItemShown),
        ev(1, 12, 83_000, Some(12), EventKind::ItemSkipped),
    ];
    let s = session_samples(&events);
    assert_eq!(s.len(), 2, "the skipped item was never rated");
    assert_eq!(
        (s[0].position, s[0].active_ms, s[0].latency_ms, s[0].item_ms),
        (1, 0, Some(3_000), 4_000)
    );
    // shown at 66 s after a 60 s pause: 6 s active; the 10 s pause while it was on screen is left out
    assert_eq!(
        (s[1].position, s[1].active_ms, s[1].shown_t_ms),
        (2, 6_000, 66_000)
    );
    assert_eq!(
        (s[1].latency_ms, s[1].item_ms, s[1].rating),
        (Some(4_000), 5_000, Rating::Again)
    );
}

fn sample(run: u128, position: u32, minute: u32, latency: u32, rating: Rating) -> SessionSample {
    SessionSample {
        run: Uid(run),
        item: Uid(u128::from(position)),
        position,
        shown_t_ms: minute * 60_000,
        active_ms: minute * 60_000,
        latency_ms: Some(latency),
        item_ms: latency + 1_000,
        rating,
    }
}

/// Three runs of 30 minutes, one item a minute: steady for 15 minutes, then slower and worse.
fn tiring() -> Vec<SessionSample> {
    let mut out = Vec::new();
    for run in 1..=3 {
        for minute in 0..30 {
            let (latency, rating) = if minute < 15 {
                (
                    3_000,
                    if minute % 10 == 9 {
                        Rating::Again
                    } else {
                        Rating::Good
                    },
                )
            } else {
                (
                    5_000,
                    if minute % 2 == 0 {
                        Rating::Again
                    } else {
                        Rating::Good
                    },
                )
            };
            out.push(sample(run, minute + 1, minute, latency, rating));
        }
    }
    out
}

#[test]
fn fatigue_is_found_where_recall_drops_and_answers_slow_down() {
    let r = fatigue_report(&tiring());
    assert_eq!(r.sessions, 3);
    assert_eq!(r.by_time.len(), 6);
    assert_eq!(r.by_position.len(), 5);
    assert_eq!(
        (r.baseline.reviews, r.baseline.recalled),
        (30, 27),
        "the first ten minutes"
    );
    assert_eq!(r.by_time[2].reviews, 15, "minutes 10 to 14 of three runs");
    let i = r.inflection.unwrap();
    assert_eq!((i.bucket, i.minutes), (3, 15));
    assert_eq!(i.items, Some(15), "15 items before minute 15 in every run");
    assert!((i.latency_rise - 2.0 / 3.0).abs() < 1e-6, "{i:?}");
    assert!(
        (i.recall_drop_points - 30.0).abs() < 1e-3,
        "90 % to 60 % (minutes 15 to 19): {i:?}"
    );
    assert_eq!(pace_rate(&r.by_time[0]), Some(1.0));
    assert_eq!(pace_rate(&r.by_time[1]), Some(0.8));
}

#[test]
fn steady_runs_have_no_fatigue_point() {
    let steady: Vec<SessionSample> = (0..40)
        .map(|m| sample(1, m + 1, m, 3_000, Rating::Good))
        .collect();
    let r = fatigue_report(&steady);
    assert_eq!(r.inflection, None);
    assert_eq!(r.by_position[4].reviews, 0, "no position 51 or later");
    assert_eq!(r.by_position[3].reviews, 5, "positions 36 to 40");
}

#[test]
fn a_spike_needs_five_answers_and_twice_the_median_plus_five_seconds() {
    let earlier = [3_000, 3_200, 2_800, 3_100, 2_900];
    assert_eq!(
        latency_spike(&earlier[..4], 20_000),
        None,
        "too few answers"
    );
    assert_eq!(
        latency_spike(&earlier, 7_000),
        None,
        "not five seconds more"
    );
    assert_eq!(latency_spike(&earlier, 8_000), Some(8_000.0 / 3_000.0));
    assert_eq!(
        latency_spike(&[400; 6], 1_000),
        None,
        "twice, but under five seconds more"
    );
}

proptest! {
    #[test]
    fn every_sample_is_in_one_time_bucket_and_one_position_bucket(
        minutes in proptest::collection::vec((0_u32..90, 1_u32..80, any::<bool>()), 0..60)
    ) {
        let samples: Vec<SessionSample> = minutes.iter().map(|(m, p, ok)| {
            sample(1, *p, *m, 2_000, if *ok { Rating::Good } else { Rating::Again })
        }).collect();
        let r = fatigue_report(&samples);
        prop_assert_eq!(r.by_time.iter().map(|b| b.reviews).sum::<u32>() as usize, samples.len());
        prop_assert_eq!(r.by_position.iter().map(|b| b.reviews).sum::<u32>() as usize, samples.len());
    }
}
