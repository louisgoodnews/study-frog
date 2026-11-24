//! The exam screens for `--screenshot`: the setup with Exam chosen, a running exam with a draft answer, and an
//! exam answered to the end (one item a frame: the reference for every other item, a wrong answer otherwise).
use crate::msg::MsgRehearsal;
use crate::msg::MsgVault;

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
        Msg::Vault(MsgVault::Navigate(Screen::Rehearse)),
        Msg::Rehearsal(MsgRehearsal::Setup(SetupEdit::Preset(
            Preset::EverythingInOrder,
        ))),
        Msg::Rehearsal(MsgRehearsal::Setup(SetupEdit::Mode(RunMode::Exam))),
        Msg::Rehearsal(MsgRehearsal::SetupStep(SetupStep::How)),
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
        "exam" if run.answer.is_empty() => {
            apply(model, Msg::Rehearsal(MsgRehearsal::AnswerEdited(text)))
        }
        "exam-result" => apply(
            model,
            Msg::Rehearsal(MsgRehearsal::RunAction(Action::SubmitAnswer(text))),
        ),
        _ => Vec::new(),
    }
}
