#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! A rating carries how long the answer was on screen and how long it should take to read, for familiarity.

mod common;

use common::views::running_model;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_rehearsal::input::Action;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

fn timing_of(fx: &[Effect]) -> Option<(u32, u32)> {
    fx.iter().find_map(|e| match e {
        Effect::RecordStep { ctx, .. } => ctx.timing.map(|t| (t.answer_ms, t.expected_ms)),
        _ => None,
    })
}

fn go(m: &mut Model, at: f64, action: Action) -> Vec<Effect> {
    m.state.now = at;
    apply(m, Msg::RunAction(action))
}

#[test]
fn a_rating_carries_the_time_the_answer_was_read_and_the_time_it_should_take() {
    let mut m = running_model(RunMode::Flip);
    go(&mut m, 0.0, Action::Start);
    go(&mut m, 2.0, Action::Flip);
    let fx = go(&mut m, 5.5, Action::Rate(Rating::Good));
    let (answer, expected) = timing_of(&fx).expect("a rating has timing");
    assert_eq!(answer, 3_500, "from revealing the answer to rating it");
    assert_eq!(expected, 252, "one word at 238 words a minute");
}

#[test]
fn only_a_rating_has_timing_and_paused_time_is_left_out() {
    let mut m = running_model(RunMode::Flip);
    let start = go(&mut m, 0.0, Action::Start);
    assert_eq!(timing_of(&start), None);
    let flip = go(&mut m, 1.0, Action::Flip);
    assert_eq!(timing_of(&flip), None, "revealing is not rating");
    go(&mut m, 2.0, Action::Pause);
    go(&mut m, 12.0, Action::Resume);
    let fx = go(&mut m, 14.0, Action::Rate(Rating::Easy));
    assert_eq!(
        timing_of(&fx).unwrap().0,
        3_000,
        "13 s since the reveal, 10 of them paused"
    );
}

#[test]
fn each_item_is_timed_on_its_own() {
    let mut m = running_model(RunMode::Flip);
    go(&mut m, 0.0, Action::Start);
    go(&mut m, 1.0, Action::Flip);
    go(&mut m, 2.0, Action::Rate(Rating::Good));
    go(&mut m, 4.0, Action::Flip);
    let fx = go(&mut m, 10.0, Action::Rate(Rating::Hard));
    assert_eq!(
        timing_of(&fx).unwrap().0,
        6_000,
        "the second item's own reveal, not the first's"
    );
}
