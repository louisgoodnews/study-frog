//! Pure transitions for classroom work: assigning a task, turning in an answer, and recording a
//! grade. A submission is locked from turn-in on; a grade is recorded once.

use crate::classroom::{
    Assignment, AssignmentKind, ClassroomError, ClassroomSession, GradeReport, GroupRoster, Owner,
    Submission,
};
use crate::classroom_ops::{check_participant, is_blank, is_student};
use crate::time::UnixMs;
use crate::uid::Uid;

/// Adds an assignment. Its id must be new, its text non-blank, and every assignee a student.
/// A group assignment needs at least one group, each with two or more distinct members, and no
/// participant may sit in two groups.
pub fn assign(
    session: &mut ClassroomSession,
    assignment: Assignment,
) -> Result<(), ClassroomError> {
    if session.assignments.iter().any(|a| a.id == assignment.id) {
        return Err(ClassroomError::DuplicateAssignment(assignment.id));
    }
    if is_blank(&assignment.title) || is_blank(&assignment.prompt) {
        return Err(ClassroomError::EmptyText);
    }
    check_kind(session, &assignment.kind)?;
    session.assignments.push(assignment);
    Ok(())
}

/// Checks the assignees of an assignment kind against the session's participants.
fn check_kind(session: &ClassroomSession, kind: &AssignmentKind) -> Result<(), ClassroomError> {
    match kind {
        AssignmentKind::Solo { assignee } => check_student(session, *assignee),
        AssignmentKind::Group { groups } => check_groups(session, groups),
    }
}

/// Checks that one participant can be assigned work.
fn check_student(session: &ClassroomSession, index: usize) -> Result<(), ClassroomError> {
    check_participant(session, index)?;
    if is_student(&session.participants[index]) {
        Ok(())
    } else {
        Err(ClassroomError::NotAStudent(index))
    }
}

/// Checks every group; `seen` carries the members already placed so none sits in two groups.
fn check_groups(session: &ClassroomSession, groups: &[GroupRoster]) -> Result<(), ClassroomError> {
    if groups.is_empty() {
        return Err(ClassroomError::GroupTooSmall);
    }
    let mut seen: Vec<usize> = Vec::new();
    for group in groups {
        check_group(session, group, &mut seen)?;
    }
    Ok(())
}

fn check_group(
    session: &ClassroomSession,
    group: &GroupRoster,
    seen: &mut Vec<usize>,
) -> Result<(), ClassroomError> {
    if is_blank(&group.name) {
        return Err(ClassroomError::EmptyText);
    }
    if group.members.len() < 2 {
        return Err(ClassroomError::GroupTooSmall);
    }
    for &member in &group.members {
        check_student(session, member)?;
        if seen.contains(&member) {
            return Err(ClassroomError::DuplicateMember(member));
        }
        seen.push(member);
    }
    Ok(())
}

/// Turns in an answer for an assignment. A solo assignee submits for themselves; a group member
/// submits for their group. Each owner can turn in once; the answer is locked from then on.
/// Returns the owner the answer belongs to.
pub fn submit(
    session: &mut ClassroomSession,
    assignment_id: Uid,
    submitter: usize,
    at: UnixMs,
    text: &str,
) -> Result<Owner, ClassroomError> {
    check_student(session, submitter)?;
    if is_blank(text) {
        return Err(ClassroomError::EmptyText);
    }
    let owner = owner_for(find_assignment(session, assignment_id)?, submitter)?;
    if find_submission(session, assignment_id, owner).is_some() {
        return Err(ClassroomError::AlreadySubmitted);
    }
    session.submissions.push(Submission {
        assignment: assignment_id,
        owner,
        text: text.to_owned(),
        submitted_at: at,
        grade: None,
    });
    Ok(owner)
}

/// The owner a student's answer is filed under for an assignment.
fn owner_for(assignment: &Assignment, submitter: usize) -> Result<Owner, ClassroomError> {
    match &assignment.kind {
        AssignmentKind::Solo { assignee } if *assignee == submitter => {
            Ok(Owner::Participant(submitter))
        }
        AssignmentKind::Group { groups } => groups
            .iter()
            .position(|g| g.members.contains(&submitter))
            .map(Owner::Group)
            .ok_or(ClassroomError::NotAssigned(submitter)),
        AssignmentKind::Solo { .. } => Err(ClassroomError::NotAssigned(submitter)),
    }
}

/// Records the grade of a turned-in answer. The owner must match the assignment's kind; the
/// answer must exist and not yet be graded; points must not exceed a positive maximum.
pub fn record_grade(
    session: &mut ClassroomSession,
    assignment_id: Uid,
    owner: Owner,
    grade: GradeReport,
) -> Result<(), ClassroomError> {
    check_owner(find_assignment(session, assignment_id)?, owner)?;
    if grade.max == 0 || grade.points > grade.max {
        return Err(ClassroomError::GradeOutOfRange {
            points: grade.points,
            max: grade.max,
        });
    }
    let index =
        find_submission(session, assignment_id, owner).ok_or(ClassroomError::NotSubmitted)?;
    let submission = &mut session.submissions[index];
    if submission.grade.is_some() {
        return Err(ClassroomError::AlreadyGraded);
    }
    submission.grade = Some(grade);
    Ok(())
}

/// Checks that an owner exists for an assignment's kind.
fn check_owner(assignment: &Assignment, owner: Owner) -> Result<(), ClassroomError> {
    match (&assignment.kind, owner) {
        (AssignmentKind::Solo { assignee }, Owner::Participant(p)) if *assignee == p => Ok(()),
        (AssignmentKind::Group { groups }, Owner::Group(g)) => {
            if g < groups.len() {
                Ok(())
            } else {
                Err(ClassroomError::UnknownGroup(g))
            }
        }
        _ => Err(ClassroomError::WrongOwner),
    }
}

fn find_assignment(session: &ClassroomSession, id: Uid) -> Result<&Assignment, ClassroomError> {
    session
        .assignments
        .iter()
        .find(|a| a.id == id)
        .ok_or(ClassroomError::UnknownAssignment(id))
}

fn find_submission(session: &ClassroomSession, assignment: Uid, owner: Owner) -> Option<usize> {
    session
        .submissions
        .iter()
        .position(|s| s.assignment == assignment && s.owner == owner)
}
