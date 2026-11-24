//! Control commands for stacks and for rehearsing: the plans for `stack` and `rehearse`.
use super::*;

pub(super) fn stack(state: &State, cmd: &StackCmd) -> Result<Plan, String> {
    Ok(match cmd {
        StackCmd::Create(title, parent) => {
            let args = NewItemArgs {
                kind: NewKind::Stack,
                title: title.clone(),
                text: String::new(),
                answer: String::new(),
                tags: String::new(),
                stack: parent.clone(),
            };
            return create(state, &args);
        }
        StackCmd::Open(folder) | StackCmd::Settings(folder) => {
            resolve_stack(state, folder)?;
            vec![open_stack(folder)]
        }
        StackCmd::Delete(folder) => return panels::stack_delete(state, folder),
        StackCmd::Move(folder, into) => {
            resolve_stack(state, folder)?;
            if !into.is_empty() {
                resolve_stack(state, into)?;
            }
            let m = crate::stack_view_ui::StackViewMsg::MoveStack(folder.clone(), into.clone());
            vec![vec![Msg::Library(LibraryMsg::Stack(m))]]
        }
    })
}

pub(super) fn rehearse(state: &State, cmd: &RehearseCmd) -> Result<Plan, String> {
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
