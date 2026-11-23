#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The run event sequence: every number is written once per run, whatever the screen still holds.

mod common;

use common::{NOW, build, drive, env, script, started};
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::review::Rating::Good;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::rehearse::recover_runs;
use studyfrog_store::runs::load_events;

fn assert_increasing(seqs: &[u32]) {
    assert!(
        seqs.windows(2).all(|w| w[0] < w[1]),
        "sequence numbers must be unique and increasing: {seqs:?}"
    );
}

/// Opening the vault again while a run is on screen appends `run_aborted` at the next free number.
/// The screen still holds its session, which counts from its own number: its next step used to
/// collide with that row (UNIQUE run_event.run_uid, seq).
#[test]
fn a_step_after_a_reopen_does_not_repeat_a_sequence_number() {
    let mut env = env();
    let mut session = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(&mut env, &mut session, NOW, vec![(0, Action::Start)]);

    assert_eq!(recover_runs(&mut env.conn).unwrap(), 1);
    drive(&mut env, &mut session, NOW, vec![(1000, Action::Flip)]);

    let seqs: Vec<u32> = load_events(&env.conn, Uid(100))
        .unwrap()
        .iter()
        .map(|e| e.seq)
        .collect();
    assert_increasing(&seqs);
    assert_eq!(seqs.len(), 4, "started, shown, run_aborted, flipped");
}

/// Quick successive ratings each write their own numbers and none is lost.
#[test]
fn quick_successive_ratings_are_all_saved_with_unique_numbers() {
    let mut env = env();
    let mut session = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(&mut env, &mut session, NOW, script(&[Good, Good, Good]));
    let seqs: Vec<u32> = load_events(&env.conn, Uid(100))
        .unwrap()
        .iter()
        .map(|e| e.seq)
        .collect();
    assert_increasing(&seqs);
}
