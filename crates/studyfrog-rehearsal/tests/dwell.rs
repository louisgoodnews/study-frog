#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Dwell is the time an item was on screen before its rating, paused time excluded; the latency is
//! the time to the first response. The two are measured apart (BUG-0048).

mod common;

use common::{go, session};
use studyfrog_core::plan::plan_sequential;
use studyfrog_core::review::Rating::Good;
use studyfrog_core::run::RunMode;
use studyfrog_rehearsal::input::Action;

#[test]
fn dwell_runs_from_the_item_shown_to_its_rating_and_excludes_pauses() {
    let mut s = session(RunMode::Flip, plan_sequential(), &[1]);
    go(&mut s, 0, Action::Start);
    go(&mut s, 1000, Action::Pause);
    go(&mut s, 4000, Action::Resume);
    let flip = go(&mut s, 5500, Action::Flip);
    assert_eq!(flip.latency.unwrap().ms, 2500, "time to the flip");
    assert_eq!(flip.dwell_ms, None, "only a rating has a dwell");
    let rated = go(&mut s, 7000, Action::Rate(Good));
    assert_eq!(rated.dwell_ms, Some(4000), "7000 - 0 - 3000 paused");
    assert!(rated.latency.is_none());
}

#[test]
fn a_skipped_item_has_no_dwell() {
    let mut s = session(RunMode::Flip, plan_sequential(), &[1, 2]);
    go(&mut s, 0, Action::Start);
    let skipped = go(&mut s, 900, Action::Skip);
    assert_eq!(skipped.dwell_ms, None);
}
