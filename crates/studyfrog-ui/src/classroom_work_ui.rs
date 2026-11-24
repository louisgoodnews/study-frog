//! The Classroom assignment reducer (TASK-0294): the open task, the drafted answer, the group thread,
//! and the requests to turn in, grade and assign group work. Each request is one effect; the rules
//! (who may write, what locks) are the service's, and the reducer only refuses what is obviously wrong.

use studyfrog_core::classroom::Owner;
use studyfrog_core::classroom_ops::is_blank;
use studyfrog_core::classroom_status::{owner_of, submission_for};
use studyfrog_core::uid::Uid;
use studyfrog_service::classroom_task::Assignees;

use crate::classroom_types::{ClassroomEffect, ClassroomMsg};
use crate::classroom_ui::{finish_request, turn, turn_effect, user_index};
use crate::effect::Effect;
use crate::state::State;

/// The students of a group task are split into groups of this many.
const GROUP_SIZE: usize = 2;

/// Advances the assignment screen by one message. Messages of other areas are not expected here.
pub fn apply(state: &mut State, msg: ClassroomMsg) -> Vec<Effect> {
    match msg {
        ClassroomMsg::OpenTask(task) => {
            state.classroom.open_task = task;
            state.classroom.error = None;
            Vec::new()
        }
        ClassroomMsg::DraftEdit(text) => {
            if let Some(id) = state.classroom.open_task {
                set_draft(&mut state.classroom.drafts, id, text);
            }
            Vec::new()
        }
        ClassroomMsg::GroupInput(text) => {
            state.classroom.group_input = text;
            Vec::new()
        }
        ClassroomMsg::SendGroup => send_group(state),
        ClassroomMsg::TurnIn => turn_in(state),
        ClassroomMsg::AskGrade => ask_grade(state),
        ClassroomMsg::AssignGroups => turn(state, |s, provider| {
            Ok(ClassroomEffect::AssignTask {
                id: s.id,
                provider,
                assignees: Assignees::Groups { size: GROUP_SIZE },
            })
        }),
        _ => Vec::new(),
    }
}

/// Writes the group composer text as the user to the open group task's thread.
fn send_group(state: &mut State) -> Vec<Effect> {
    if state.classroom.busy {
        return Vec::new();
    }
    let text = state.classroom.group_input.trim().to_owned();
    if is_blank(&text) {
        state.classroom.error = Some("Type a message to your group first.".to_owned());
        return Vec::new();
    }
    let open = state.classroom.open_task;
    let outcome = turn_effect(state, |s, provider| {
        let assignment = open.ok_or("Open a group task first.")?;
        let user = user_index(s).ok_or("This session has no user to speak for.")?;
        let Some(Owner::Group(group)) = owner_of(s, assignment, user) else {
            return Err("Only a member of a group task can write to its group.".to_owned());
        };
        Ok(ClassroomEffect::GroupMessage {
            id: s.id,
            provider,
            assignment,
            group,
            text,
        })
    });
    if outcome.is_ok() {
        state.classroom.group_input.clear();
    }
    finish_request(state, outcome)
}

/// Turns in the drafted answer for the open task as the user. The draft must not be blank.
fn turn_in(state: &mut State) -> Vec<Effect> {
    if state.classroom.busy {
        return Vec::new();
    }
    let open = state.classroom.open_task;
    let draft = open
        .map(|id| draft_of(&state.classroom.drafts, id).to_owned())
        .unwrap_or_default();
    let outcome = turn_effect(state, |s, provider| {
        let assignment = open.ok_or("Open a task first.")?;
        let user = user_index(s).ok_or("This session has no user to turn in for.")?;
        let owner = owner_of(s, assignment, user).ok_or("This task is not yours to turn in.")?;
        if submission_for(s, assignment, owner).is_some() {
            return Err("This answer is already turned in.".to_owned());
        }
        if is_blank(&draft) {
            return Err("Write your answer before you turn it in.".to_owned());
        }
        Ok(ClassroomEffect::TurnIn {
            id: s.id,
            provider,
            assignment,
            submitter: user,
            text: draft.trim().to_owned(),
        })
    });
    finish_request(state, outcome)
}

/// Asks the teacher to grade the user's turned-in answer to the open task.
fn ask_grade(state: &mut State) -> Vec<Effect> {
    if state.classroom.busy {
        return Vec::new();
    }
    let open = state.classroom.open_task;
    let outcome = turn_effect(state, |s, provider| {
        let assignment = open.ok_or("Open a task first.")?;
        let user = user_index(s).ok_or("This session has no user to grade.")?;
        let owner = owner_of(s, assignment, user).ok_or("This task is not yours to grade.")?;
        match submission_for(s, assignment, owner) {
            None => Err("Turn in your answer before the teacher grades it.".to_owned()),
            Some(sub) if sub.grade.is_some() => Err("This answer is already graded.".to_owned()),
            Some(_) => Ok(ClassroomEffect::Grade {
                id: s.id,
                provider,
                assignment,
                owner,
            }),
        }
    });
    finish_request(state, outcome)
}

/// Sets the drafted answer for one assignment, adding its slot the first time.
fn set_draft(drafts: &mut Vec<(Uid, String)>, id: Uid, text: String) {
    match drafts.iter_mut().find(|(d, _)| *d == id) {
        Some(slot) => slot.1 = text,
        None => drafts.push((id, text)),
    }
}

/// The drafted answer for an assignment; empty when nothing was drafted.
pub fn draft_of(drafts: &[(Uid, String)], id: Uid) -> &str {
    drafts
        .iter()
        .find(|(d, _)| *d == id)
        .map_or("", |(_, text)| text.as_str())
}
