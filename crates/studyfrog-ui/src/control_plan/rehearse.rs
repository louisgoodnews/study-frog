//! The clicks `rehearse ...` stands for: setup, start, the reveal, ratings, marks and the typed or
//! chosen answer (STORY-0222). Pure: the state gives the messages a user's clicks would send.

use studyfrog_rehearsal::input::Action;

use crate::control_cmd::RehearseCmd;
use crate::control_plan::Plan;
use crate::msg::{Msg, MsgRehearsal, MsgVault};
use crate::run_update::choice::{ChoiceMsg, check_choose, is_pending};
use crate::state::{Screen, State};

pub fn plan(state: &State, cmd: &RehearseCmd) -> Result<Plan, String> {
    crate::control_exam::check(state, cmd)?;
    let act = |a| vec![vec![Msg::Rehearsal(MsgRehearsal::RunAction(a))]];
    Ok(match cmd {
        RehearseCmd::Setup(args) => vec![crate::control_setup::setup_msgs(state, args)?],
        RehearseCmd::Start => vec![vec![
            Msg::Vault(MsgVault::Navigate(Screen::Rehearse)),
            Msg::Rehearsal(MsgRehearsal::StartRunClicked),
        ]],
        RehearseCmd::Flip => act(Action::Flip),
        RehearseCmd::Answer(text) => vec![vec![
            Msg::Rehearsal(MsgRehearsal::AnswerEdited(text.clone())),
            Msg::Rehearsal(MsgRehearsal::RunAction(Action::SubmitAnswer(text.clone()))),
        ]],
        RehearseCmd::Rate(r) => act(Action::Rate(*r)),
        RehearseCmd::Mark(d) => act(Action::Mark(*d)),
        RehearseCmd::Skip => act(Action::Skip),
        RehearseCmd::Pause => act(Action::Pause),
        RehearseCmd::Resume => act(Action::Resume),
        RehearseCmd::Stop => act(Action::Abort),
        RehearseCmd::Leave | RehearseCmd::Finish => {
            vec![vec![Msg::Rehearsal(MsgRehearsal::LeaveRun)]]
        }
        RehearseCmd::Type(text) => vec![vec![Msg::Rehearsal(MsgRehearsal::AnswerEdited(
            text.clone(),
        ))]],
        RehearseCmd::Choose(numbers) => {
            let indices = check_choose(state, numbers)?;
            vec![vec![Msg::Rehearsal(MsgRehearsal::Choice(ChoiceMsg::Set(
                indices,
            )))]]
        }
        RehearseCmd::Submit if is_pending(state) => {
            vec![vec![Msg::Rehearsal(MsgRehearsal::Choice(ChoiceMsg::Submit))]]
        }
        RehearseCmd::Submit => {
            let text = state
                .rehearse
                .run
                .as_ref()
                .map(|r| r.answer.clone())
                .unwrap_or_default();
            act(Action::SubmitAnswer(text))
        }
    })
}
