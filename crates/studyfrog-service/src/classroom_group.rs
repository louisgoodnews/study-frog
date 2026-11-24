//! Teammate contributions in a group task (STORY-0210, TASK-0291): before a group turns in, each
//! simulated teammate adds one contribution to the group thread. A contribution is a chat turn of that
//! student, stored like any other turn, written before the next teammate is asked.

use studyfrog_agent::lecture_prompt::Line;
use studyfrog_agent::lecture_teammate::teammate_messages;
use studyfrog_core::classroom::{AssignmentKind, ClassroomSession, Owner, Participant};
use studyfrog_core::classroom_ops::participant_name;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;

use crate::classroom_context::{Lecture, is_kind, load, model_error};
use crate::classroom_model::{Model, check_consent};
use crate::classroom_turns::store_turn;
use crate::session::ServiceError;

/// The group's task and its members, read from the session.
struct GroupWork<'a> {
    title: &'a str,
    prompt: &'a str,
    members: Vec<usize>,
}

/// Each simulated teammate of group `group` adds one contribution to the group thread, in roster order,
/// before the group turns in. Returns the contributions in the order they were made. Refused, with no
/// model call, when the assignment is not a group task, the group does not exist, or the group has
/// already turned in.
pub fn group_contributions(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    at: UnixMs,
    assignment: Uid,
    group: usize,
) -> Result<Vec<String>, ServiceError> {
    check_consent(lecture.provider)?;
    let teammates = simulated_teammates(&load(lecture)?, assignment, group)?;
    let mut out = Vec::with_capacity(teammates.len());
    for student in teammates {
        out.push(contribute(lecture, model, at, assignment, group, student)?);
    }
    Ok(out)
}

/// One teammate's contribution: asked from the thread as it is now, then stored.
fn contribute(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    at: UnixMs,
    assignment: Uid,
    group: usize,
    student: usize,
) -> Result<String, ServiceError> {
    let mut session = load(lecture)?;
    let text = ask_teammate(&session, model, assignment, group, student)?;
    store_turn(lecture, &mut session, student, at, &text)?;
    Ok(text.trim().to_owned())
}

fn ask_teammate(
    session: &ClassroomSession,
    model: &mut Model<'_>,
    assignment: Uid,
    group: usize,
    student: usize,
) -> Result<String, ServiceError> {
    let work = group_work(session, assignment, group)?;
    let persona = crate::classroom_context::persona_of(session, student)?;
    let thread = thread_of(session, &work.members);
    let messages = teammate_messages(&persona, work.title, work.prompt, &thread);
    model(&messages).map_err(model_error)
}

/// The simulated students in the group, in roster order. The user is a member but never answered by
/// the model.
fn simulated_teammates(
    session: &ClassroomSession,
    assignment: Uid,
    group: usize,
) -> Result<Vec<usize>, ServiceError> {
    let work = group_work(session, assignment, group)?;
    Ok(work
        .members
        .into_iter()
        .filter(|&m| {
            is_kind(session, m, |p| {
                matches!(p, Participant::SimulatedStudent { .. })
            })
        })
        .collect())
}

/// The group's task, checked: it must exist, be a group task, have this group, and not be turned in.
fn group_work(
    session: &ClassroomSession,
    assignment: Uid,
    group: usize,
) -> Result<GroupWork<'_>, ServiceError> {
    let task = session
        .assignments
        .iter()
        .find(|a| a.id == assignment)
        .ok_or_else(|| ServiceError::Invalid("no task has that id".to_owned()))?;
    let AssignmentKind::Group { groups } = &task.kind else {
        return Err(ServiceError::Invalid(
            "only a group task has teammate turns".to_owned(),
        ));
    };
    let roster = groups.get(group).ok_or_else(|| {
        ServiceError::Invalid("the task has no group with that number".to_owned())
    })?;
    let turned_in = session
        .submissions
        .iter()
        .any(|s| s.assignment == assignment && s.owner == Owner::Group(group));
    if turned_in {
        return Err(ServiceError::Invalid(
            "the group has already turned in".to_owned(),
        ));
    }
    Ok(GroupWork {
        title: &task.title,
        prompt: &task.prompt,
        members: roster.members.clone(),
    })
}

/// The chat turns by the group's members, oldest first, with their names.
fn thread_of<'a>(session: &'a ClassroomSession, members: &[usize]) -> Vec<Line<'a>> {
    session
        .turns
        .iter()
        .filter(|t| members.contains(&t.speaker))
        .map(|t| Line {
            speaker: session
                .participants
                .get(t.speaker)
                .map_or("?", participant_name),
            text: &t.text,
        })
        .collect()
}
