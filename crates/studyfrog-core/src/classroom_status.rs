//! Where classroom work stands: assigned, submitted or graded. Pure lookups over a session; nothing
//! here changes it. The UI shows these, so the rules for owners and status live in one place.

use crate::classroom::{AssignmentKind, ClassroomSession, GradeReport, Owner, Submission};
use crate::uid::Uid;

/// How far one piece of work has got. Declared in order, so a later status is the more advanced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkStatus {
    /// Set but not turned in.
    Assigned,
    /// Turned in and locked, not yet graded.
    Submitted,
    /// Turned in and graded.
    Graded,
}

/// The owner a participant files work under for an assignment, if they have any: their own solo
/// work, or the group they sit in.
pub fn owner_of(session: &ClassroomSession, assignment: Uid, participant: usize) -> Option<Owner> {
    let task = session.assignments.iter().find(|a| a.id == assignment)?;
    match &task.kind {
        AssignmentKind::Solo { assignee } if *assignee == participant => {
            Some(Owner::Participant(participant))
        }
        AssignmentKind::Solo { .. } => None,
        AssignmentKind::Group { groups } => groups
            .iter()
            .position(|g| g.members.contains(&participant))
            .map(Owner::Group),
    }
}

/// Every owner an assignment has: the assignee of a solo task, or each group of a group task.
pub fn owners_of(session: &ClassroomSession, assignment: Uid) -> Vec<Owner> {
    let Some(task) = session.assignments.iter().find(|a| a.id == assignment) else {
        return Vec::new();
    };
    match &task.kind {
        AssignmentKind::Solo { assignee } => vec![Owner::Participant(*assignee)],
        AssignmentKind::Group { groups } => (0..groups.len()).map(Owner::Group).collect(),
    }
}

/// The turned-in answer of `owner` for an assignment, if there is one.
pub fn submission_for(
    session: &ClassroomSession,
    assignment: Uid,
    owner: Owner,
) -> Option<&Submission> {
    session
        .submissions
        .iter()
        .find(|s| s.assignment == assignment && s.owner == owner)
}

/// The grade of `owner`'s answer, if it has been graded.
pub fn grade_for(
    session: &ClassroomSession,
    assignment: Uid,
    owner: Owner,
) -> Option<&GradeReport> {
    submission_for(session, assignment, owner).and_then(|s| s.grade.as_ref())
}

/// The status of one owner's work.
pub fn status_of(session: &ClassroomSession, assignment: Uid, owner: Owner) -> WorkStatus {
    match submission_for(session, assignment, owner) {
        None => WorkStatus::Assigned,
        Some(s) if s.grade.is_some() => WorkStatus::Graded,
        Some(_) => WorkStatus::Submitted,
    }
}

/// The status an assignment shows for the user: the user's own work when the user has some, else
/// the least advanced of all its owners, so a task is only "graded" once every owner is graded.
pub fn task_status(session: &ClassroomSession, assignment: Uid, user: Option<usize>) -> WorkStatus {
    let own = user.and_then(|u| owner_of(session, assignment, u));
    let owners = match own {
        Some(owner) => vec![owner],
        None => owners_of(session, assignment),
    };
    owners
        .into_iter()
        .map(|owner| status_of(session, assignment, owner))
        .min()
        .unwrap_or(WorkStatus::Assigned)
}
