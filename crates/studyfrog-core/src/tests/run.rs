use std::collections::HashSet;

use crate::event::{EventKind, event_name, is_item_event};
use crate::review::Rating;
use crate::run::{
    RunMode, RunState, run_mode_from_name, run_mode_name, run_state_from_name, run_state_name,
};

#[test]
fn mode_and_state_names_round_trip() {
    for mode in [RunMode::Flip, RunMode::Answer] {
        assert_eq!(run_mode_from_name(run_mode_name(mode)), Some(mode));
    }
    for state in [
        RunState::Idle,
        RunState::Running,
        RunState::Paused,
        RunState::Finished,
        RunState::Aborted,
    ] {
        assert_eq!(run_state_from_name(run_state_name(state)), Some(state));
    }
    assert_eq!(run_state_from_name("bogus"), None);
}

#[test]
fn event_names_are_unique_and_item_events_are_classified() {
    let kinds = [
        EventKind::RunStarted,
        EventKind::ItemShown,
        EventKind::ItemSkipped,
        EventKind::ItemFlipped,
        EventKind::AnswerSubmitted(String::new()),
        EventKind::ItemRated(Rating::Good),
        EventKind::RunPaused,
        EventKind::RunResumed,
        EventKind::RunFinished,
        EventKind::RunAborted,
    ];
    let names: HashSet<_> = kinds.iter().map(event_name).collect();
    assert_eq!(names.len(), kinds.len());
    assert_eq!(kinds.iter().filter(|k| is_item_event(k)).count(), 5);
}
