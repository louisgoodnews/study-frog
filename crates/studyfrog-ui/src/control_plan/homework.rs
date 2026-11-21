//! The clicks `homework ...` stands for: the dialog filled in and saved, a status tab, advancing an
//! item, ticking a step. Pure: the state gives the stack a course folder names.

use crate::control_cmd::{HomeworkCmd, HomeworkNew};
use crate::control_plan::Plan;
use crate::homework_form::HomeworkEdit;
use crate::homework_msg::HomeworkMsg;
use crate::msg::Msg;
use crate::state::State;

fn edit(e: HomeworkEdit) -> Msg {
    Msg::Homework(HomeworkMsg::Edit(e))
}

/// The fields of a new item, in the dialog's order.
fn fields(state: &State, a: &HomeworkNew) -> Result<Vec<Msg>, String> {
    let mut out = vec![
        Msg::Homework(HomeworkMsg::New),
        edit(HomeworkEdit::Title(a.title.clone())),
    ];
    if let Some(d) = &a.due_date {
        out.push(edit(HomeworkEdit::DueDate(d.clone())));
    }
    if let Some(t) = &a.due_time {
        out.push(edit(HomeworkEdit::DueTime(t.clone())));
    }
    if let Some(m) = a.estimate {
        out.push(edit(HomeworkEdit::Estimate(m)));
    }
    if let Some(p) = a.priority {
        out.push(edit(HomeworkEdit::Priority(p)));
    }
    if let Some(folder) = &a.stack {
        let uid = state
            .vault
            .as_ref()
            .and_then(|v| v.overview.stacks.iter().find(|s| s.folder == *folder))
            .map(|s| s.uid)
            .ok_or_else(|| format!("no stack in folder {folder}"))?;
        out.push(edit(HomeworkEdit::Course(Some(uid))));
    }
    if !a.checklist.is_empty() {
        out.push(edit(HomeworkEdit::Checklist(a.checklist.join("\n"))));
    }
    Ok(out)
}

/// The plan for a homework command. A new item is filled in first, then saved in a second phase, so
/// the save sees every field.
pub fn plan(state: &State, cmd: &HomeworkCmd) -> Result<Plan, String> {
    let one = |m: HomeworkMsg| Ok(vec![vec![Msg::Homework(m)]]);
    match cmd {
        HomeworkCmd::List => Ok(Vec::new()),
        HomeworkCmd::Tab(tab) => one(HomeworkMsg::Tab(*tab)),
        HomeworkCmd::Advance(id) => one(HomeworkMsg::Advance(*id)),
        HomeworkCmd::Tick(id, n) => one(HomeworkMsg::ToggleStep(*id, n - 1)),
        HomeworkCmd::New(args) => Ok(vec![
            fields(state, args)?,
            vec![Msg::Homework(HomeworkMsg::Save)],
        ]),
    }
}
