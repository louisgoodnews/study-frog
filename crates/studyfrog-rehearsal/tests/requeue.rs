#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Requeue on Again and its cap.

mod common;

use common::{go, session};
use studyfrog_core::event::EventKind;
use studyfrog_core::plan::{plan_sequential, plan_spaced};
use studyfrog_core::review::Rating::{self, Again, Good};
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::session::Session;

fn shown_items(s: &mut Session, ratings: &[Rating]) -> Vec<u128> {
    let mut shown = Vec::new();
    let mut t = 0;
    let mut out = go(s, t, Action::Start);
    for r in ratings {
        shown.extend(
            out.events
                .iter()
                .filter(|e| e.kind == EventKind::ItemShown)
                .filter_map(|e| e.item.map(|u| u.0)),
        );
        t += 10;
        go(s, t, Action::Flip);
        out = go(s, t + 1, Action::Rate(*r));
    }
    shown.extend(
        out.events
            .iter()
            .filter(|e| e.kind == EventKind::ItemShown)
            .filter_map(|e| e.item.map(|u| u.0)),
    );
    shown
}

#[test]
fn again_returns_to_the_end_of_the_queue_up_to_three_times() {
    let mut s = session(RunMode::Flip, plan_spaced(), &[1, 2]);
    assert_eq!(shown_items(&mut s, &[Again, Good, Good]), [1, 2, 1]);
    assert_eq!(s.run.state, RunState::Finished);

    let mut stubborn = session(RunMode::Flip, plan_spaced(), &[1]);
    // Item 1 is rated Again five times; it returns only three times (four showings in total).
    assert_eq!(
        shown_items(&mut stubborn, &[Again, Again, Again, Again]),
        [1, 1, 1, 1]
    );
    assert_eq!(stubborn.run.state, RunState::Finished);
}

#[test]
fn plans_without_requeue_and_skips_never_requeue() {
    let mut s = session(RunMode::Flip, plan_sequential(), &[1, 2]);
    assert_eq!(shown_items(&mut s, &[Again, Again]), [1, 2]);
    let mut k = session(RunMode::Flip, plan_spaced(), &[1, 2]);
    go(&mut k, 0, Action::Start);
    go(&mut k, 1, Action::Skip);
    go(&mut k, 2, Action::Skip);
    assert_eq!(k.run.state, RunState::Finished);
    assert_eq!(k.queue.len(), 2);
}
