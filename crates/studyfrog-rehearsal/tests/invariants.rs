#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Invariants of the state machine over arbitrary input sequences.

mod common;

use common::{at, session};
use proptest::prelude::*;
use studyfrog_core::event::EventKind;
use studyfrog_core::plan::{RunPlan, plan_spaced};
use studyfrog_core::review::Rating::{Again, Good};
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::step::step;

fn action() -> impl Strategy<Value = Action> {
    prop_oneof![
        Just(Action::Start),
        Just(Action::Flip),
        Just(Action::Skip),
        Just(Action::Pause),
        Just(Action::Resume),
        Just(Action::Abort),
        Just(Action::Rate(Again)),
        Just(Action::Rate(Good)),
        "[a-z]{0,6}".prop_map(Action::SubmitAnswer),
    ]
}

proptest! {
    #[test]
    fn any_input_sequence_keeps_the_invariants(
        mode in prop_oneof![Just(RunMode::Flip), Just(RunMode::Answer)],
        requeue in any::<bool>(),
        queue in prop::collection::vec(1_u128..6, 0..6),
        steps in prop::collection::vec((0_u32..2000, action()), 0..60),
    ) {
        let plan = RunPlan { requeue_again: requeue, ..plan_spaced() };
        let mut s = session(mode, plan, &queue);
        let (mut t, mut seqs, mut terminal) = (0_u32, Vec::new(), 0);
        for (dt, a) in steps {
            t += dt;
            let was_over = matches!(s.run.state, RunState::Finished | RunState::Aborted);
            if let Ok(out) = step(&mut s, at(t, a)) {
                prop_assert!(!was_over, "accepted input after the run ended");
                seqs.extend(out.events.iter().map(|e| e.seq));
                terminal += out
                    .events
                    .iter()
                    .filter(|e| matches!(e.kind, EventKind::RunFinished | EventKind::RunAborted))
                    .count();
            }
            prop_assert!(s.cursor <= s.queue.len());
        }
        prop_assert_eq!(seqs, (0..s.next_seq).collect::<Vec<_>>());
        prop_assert!(terminal <= 1);
        prop_assert_eq!(terminal == 1, matches!(s.run.state, RunState::Finished | RunState::Aborted));
    }
}
