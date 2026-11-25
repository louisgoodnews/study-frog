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
