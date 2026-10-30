#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The end-of-run summary.

use studyfrog_core::event::{EventKind, Latency, LatencyKind, RunEvent};
use studyfrog_core::review::Rating::{Again, Easy, Good, Hard};
use studyfrog_core::uid::Uid;
use studyfrog_ui::summary::{Summary, summarize};

fn ev(seq: u32, t_ms: u32, kind: EventKind) -> RunEvent {
    RunEvent {
        run: Uid(1),
        seq,
        t_ms,
        item: Some(Uid(u128::from(seq))),
        kind,
    }
}

fn lat(kind: LatencyKind, ms: u32) -> Latency {
    Latency {
        item: Uid(1),
        kind,
        ms,
    }
}

#[test]
fn counts_ratings_skips_answers_and_the_median_latency() {
    let events = [
        ev(0, 0, EventKind::RunStarted),
        ev(1, 900, EventKind::ItemRated(Good)),
        ev(2, 1500, EventKind::ItemRated(Good)),
        ev(3, 2000, EventKind::ItemRated(Again)),
        ev(4, 2500, EventKind::ItemRated(Easy)),
        ev(5, 2600, EventKind::ItemRated(Hard)),
        ev(6, 3000, EventKind::ItemSkipped),
        ev(7, 4200, EventKind::AnswerSubmitted("x".into())),
        ev(8, 4200, EventKind::RunFinished),
    ];
    let latencies = [
        lat(LatencyKind::Flip, 3000),
        lat(LatencyKind::Flip, 1000),
        lat(LatencyKind::Answer, 2000),
        lat(LatencyKind::Skip, 9),
    ];
    let s = summarize(&events, &latencies);
    assert_eq!(s.rated, [1, 1, 2, 1]);
    assert_eq!((s.skipped, s.answered, s.total_ms), (1, 1, 4200));
    assert_eq!(
        s.median_latency_ms,
        Some(2000),
        "skips are not recall attempts"
    );
}

#[test]
fn an_empty_run_summarises_to_nothing() {
    assert_eq!(summarize(&[], &[]), Summary::default());
    let two = [lat(LatencyKind::Flip, 100), lat(LatencyKind::Flip, 300)];
    assert_eq!(
        summarize(&[], &two).median_latency_ms,
        Some(100),
        "the lower middle value"
    );
}
