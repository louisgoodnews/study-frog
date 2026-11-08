#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! A run that was played and ended, for the review tests.

use studyfrog_core::event::Difficulty;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::content::ItemContent;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::review::current_mark;

use super::views::running_model;

pub const TWENTY_WORDS: &str = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";

pub fn act(m: &mut Model, now: f64, action: Action) -> Vec<Effect> {
    m.state.now = now;
    apply(m, Msg::RunAction(action))
}

pub fn mark_of(m: &Model) -> Option<Difficulty> {
    current_mark(m.state.rehearse.run.as_ref().unwrap())
}

/// Item 1 has a twenty-word back and was reviewed before; it is read for `read_s` seconds.
pub fn finished_run(read_s: f64, previous: Option<Rating>, rating: Rating) -> Model {
    finished_run_marked(read_s, previous, rating, Difficulty::Medium)
}

pub fn finished_run_marked(
    read_s: f64,
    previous: Option<Rating>,
    rating: Rating,
    mark: Difficulty,
) -> Model {
    let mut m = running_model(RunMode::Flip);
    let content = ItemContent {
        title: "Numbers".into(),
        prompt: "Count".into(),
        answer: TWENTY_WORDS.into(),
        previous,
        source: None,
        color: None,
        expected: None,
    };
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(content)));
    act(&mut m, 0.5, Action::Mark(mark));
    act(&mut m, 1.0, Action::Flip);
    act(&mut m, 1.0 + read_s, Action::Rate(rating));
    act(&mut m, 20.0, Action::Abort);
    m
}
