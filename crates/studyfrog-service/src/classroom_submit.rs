//! Turn-in and simulated peer answers (STORY-0210, TASK-0292). A turn-in locks the answer in the store;
//! a group task can also record the answers of the simulated groups it has beside the user's group, for
//! comparison. Each use case writes the session once, after every change in it succeeded.

use studyfrog_agent::lecture_peer::peer_messages;
use studyfrog_core::classroom::{AssignmentKind, ClassroomSession, Owner, Participant};
use studyfrog_core::classroom_ops::participant_name;
use studyfrog_core::classroom_work::submit;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;

use crate::classroom_context::{
    Lecture, classroom_error, find_assignment, is_kind, load, model_error, persist, submission_of,
};
use crate::classroom_model::{Model, check_consent};
use crate::session::ServiceError;

/// A simulated group that has not turned in: its index among the groups and its members.
struct Peer {
    group: usize,
    members: Vec<usize>,
}

/// Turns in `text` for `assignment` as `submitter`: the assignee of a solo task, or a member of a group.
/// The answer is trimmed, stored and locked; a second turn-in is refused and the stored answer is kept.
/// Returns the owner the answer is filed under. No model is called.
pub fn turn_in(
    lecture: &Lecture<'_>,
    at: UnixMs,
    assignment: Uid,
    submitter: usize,
    text: &str,
) -> Result<Owner, ServiceError> {
    let mut session = load(lecture)?;
    let owner =
        submit(&mut session, assignment, submitter, at, text.trim()).map_err(classroom_error)?;
    persist(lecture, &session)?;
    Ok(owner)
}

/// Records one answer for each simulated group of a group task: a group with no user in it that has not
/// turned in yet. The model writes each answer. The session is written once, after every answer exists,
/// so when the model fails for any group nothing is stored and the error is returned. Returns the owners
/// recorded, in group order. A solo task has no simulated peers, so nothing is recorded and no model is
/// called. Refused before any model call when the provider may not receive vault text.
pub fn record_peer_answers(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    at: UnixMs,
    assignment: Uid,
) -> Result<Vec<Owner>, ServiceError> {
    check_consent(lecture.provider)?;
    let mut session = load(lecture)?;
    let peers = peers_of(&session, assignment)?;
    if peers.is_empty() {
        return Ok(Vec::new());
    }
    let mut owners = Vec::with_capacity(peers.len());
    for peer in peers {
        let Some(&submitter) = peer.members.first() else {
            continue;
        };
        let text = answer_for(&session, model, assignment, &peer.members)?;
        submit(&mut session, assignment, submitter, at, &text).map_err(classroom_error)?;
        owners.push(Owner::Group(peer.group));
    }
    persist(lecture, &session)?;
    Ok(owners)
}

/// The simulated groups of a group task that have not turned in. A solo task has none.
fn peers_of(session: &ClassroomSession, assignment: Uid) -> Result<Vec<Peer>, ServiceError> {
    let task = find_assignment(session, assignment)?;
    let AssignmentKind::Group { groups } = &task.kind else {
        return Ok(Vec::new());
    };
    Ok(groups
        .iter()
        .enumerate()
        .filter(|(_, roster)| !roster.members.iter().any(|&m| is_user(session, m)))
        .filter(|(group, _)| submission_of(session, assignment, Owner::Group(*group)).is_none())
        .map(|(group, roster)| Peer {
            group,
            members: roster.members.clone(),
        })
        .collect())
}

fn is_user(session: &ClassroomSession, index: usize) -> bool {
    is_kind(session, index, |p| matches!(p, Participant::User { .. }))
}

/// The answer a simulated group writes, from the model's reply. A blank reply is refused.
fn answer_for(
    session: &ClassroomSession,
    model: &mut Model<'_>,
    assignment: Uid,
    members: &[usize],
) -> Result<String, ServiceError> {
    let task = find_assignment(session, assignment)?;
    let names: Vec<&str> = members
        .iter()
        .filter_map(|&m| session.participants.get(m))
        .map(participant_name)
        .collect();
    let messages = peer_messages(&task.title, &task.prompt, &task.rubric, &names);
    let reply = model(&messages).map_err(model_error)?;
    let text = reply.trim();
    if text.is_empty() {
        return Err(ServiceError::Invalid(
            "the model gave a blank answer for a simulated group".to_owned(),
        ));
    }
    Ok(text.to_owned())
}
