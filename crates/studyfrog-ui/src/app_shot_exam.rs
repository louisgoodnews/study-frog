//! The exam screens for `--screenshot`: the setup with Exam chosen, a running exam with a draft answer, and an
//! exam answered to the end (one item a frame: the reference for every other item, a wrong answer otherwise).

use studyfrog_core::run::RunMode;
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::session::current_item;

use crate::effect::Effect;
use crate::model::{Model, apply};
use crate::msg::Msg;
use crate::run_update::awaiting_reveal;
use crate::setup::{Preset, SetupEdit, SetupStep};
use crate::state::Screen;

/// Rehearse, everything in order, Exam, on the "How" step.
pub fn setup(model: &mut Model) -> Vec<Effect> {
    [
        Msg::Navigate(Screen::Rehearse),
        Msg::Setup(SetupEdit::Preset(Preset::EverythingInOrder)),
        Msg::Setup(SetupEdit::Mode(RunMode::Exam)),
        Msg::SetupStep(SetupStep::How),
    ]
    .into_iter()
    .flat_map(|m| apply(model, m))
    .collect()
}

/// One frame of answering: a draft on `exam`, a submitted answer on `exam-result`.
pub fn play(model: &mut Model, name: &str) -> Vec<Effect> {
    let Some(run) = &model.state.rehearse.run else {
        return Vec::new();
    };
    let item = current_item(&run.session).filter(|_| awaiting_reveal(run));
    let Some(content) = item.and_then(|i| run.content.get(&i)) else {
        return Vec::new();
    };
    let right = run.session.cursor % 2 == 0;
    let text = if right {
        content.answer.clone()
    } else {
        "I do not remember".to_owned()
    };
    match name {
        "exam" if run.answer.is_empty() => apply(model, Msg::AnswerEdited(text)),
        "exam-result" => apply(model, Msg::RunAction(Action::SubmitAnswer(text))),
        _ => Vec::new(),
    }
}
