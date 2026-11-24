//! Classroom tasks (STORY-0210, TASK-0291): the teacher sets one structured task for a single student
//! or for groups made of the user and the simulated students, and the task is stored. The model's JSON
//! reply is used when it parses; otherwise a fixed template about the current segment is used.

use studyfrog_agent::lecture_groups::partition_groups;
use studyfrog_agent::lecture_task::{
    GeneratedTask, TaskMode, fallback_task, parse_task, task_messages,
};
use studyfrog_core::classroom::{Assignment, AssignmentKind, ClassroomSession, GroupRoster};
use studyfrog_core::classroom_ops::{check_participant, is_student, participant_name};
use studyfrog_core::classroom_work::assign;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::compose_ulid;

use crate::classroom_context::{
    Lecture, classroom_error, excerpts_from, fit, lines_of, load, model_error, persist, segment_of,
    teacher_index,
};
use crate::classroom_material::lecture_material;
use crate::classroom_model::{Model, check_consent};
use crate::create::IdSource;
use crate::session::ServiceError;

/// Who a new task is set for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assignees {
    /// One student, by participant index.
    Solo(usize),
    /// Every student (the user and the simulated students) split into groups of `size`.
    Groups {
        /// The number of members a group aims for; a lone remainder joins the last group.
        size: usize,
    },
}

/// Asks the teacher for one task for `assignees`, stores it and returns its id. The id comes from
/// `ids`. Refused before any text is sent when the provider may not receive vault text, and before
/// the model is called when the assignee is not a student or no group can be made. A reply that does
/// not parse gives the fixed fallback task.
pub fn assign_task(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    ids: IdSource,
    assignees: Assignees,
) -> Result<Uid, ServiceError> {
    check_consent(lecture.provider)?;
    let mut session = load(lecture)?;
    let kind = kind_of(&session, assignees)?;
    let generated = generate(lecture, &session, model, mode_of(assignees))?;
    let id = compose_ulid(ids.now_ms, ids.random);
    let assignment = Assignment {
        id,
        title: generated.title,
        prompt: generated.prompt,
        rubric: generated.rubric,
        kind,
    };
    assign(&mut session, assignment).map_err(classroom_error)?;
    persist(lecture, &session)?;
    Ok(id)
}

fn mode_of(assignees: Assignees) -> TaskMode {
    match assignees {
        Assignees::Solo(_) => TaskMode::Solo,
        Assignees::Groups { .. } => TaskMode::Group,
    }
}

/// The assignment kind for the assignees, checked against the session before any model call.
fn kind_of(
    session: &ClassroomSession,
    assignees: Assignees,
) -> Result<AssignmentKind, ServiceError> {
    match assignees {
        Assignees::Solo(index) => solo_kind(session, index),
        Assignees::Groups { size } => group_kind(session, size),
    }
}

fn solo_kind(session: &ClassroomSession, index: usize) -> Result<AssignmentKind, ServiceError> {
    check_participant(session, index).map_err(classroom_error)?;
    if is_student(&session.participants[index]) {
        Ok(AssignmentKind::Solo { assignee: index })
    } else {
        Err(ServiceError::Invalid(
            "only a student can be given work".to_owned(),
        ))
    }
}

/// Splits the students (the user and the simulated students, never the teacher) into named groups.
fn group_kind(session: &ClassroomSession, size: usize) -> Result<AssignmentKind, ServiceError> {
    let students: Vec<usize> = session
        .participants
        .iter()
        .enumerate()
        .filter(|(_, p)| is_student(p))
        .map(|(index, _)| index)
        .collect();
    let groups = partition_groups(&students, size);
    if groups.is_empty() {
        return Err(ServiceError::Invalid(
            "a group task needs groups of at least two students".to_owned(),
        ));
    }
    Ok(AssignmentKind::Group {
        groups: groups
            .into_iter()
            .enumerate()
            .map(|(n, members)| GroupRoster {
                name: format!("Group {}", n + 1),
                members,
            })
            .collect(),
    })
}

/// The task the model proposes for `mode`, or the fallback when its reply does not parse. A model that
/// cannot be reached is an error, not a fallback.
fn generate(
    lecture: &Lecture<'_>,
    session: &ClassroomSession,
    model: &mut Model<'_>,
    mode: TaskMode,
) -> Result<GeneratedTask, ServiceError> {
    let material = lecture_material(lecture.vault, &session.sources)?;
    let excerpts = excerpts_from(&material, segment_of(session));
    let topic = excerpts
        .first()
        .and_then(|piece| piece.lines().next())
        .unwrap_or("");
    let lines = lines_of(session);
    let packed = fit(&excerpts, &lines);
    let teacher = teacher_index(session).and_then(|i| session.participants.get(i));
    let name = teacher.map_or("the teacher", participant_name);
    let messages = task_messages(name, &packed.excerpts, &packed.turns, mode);
    let reply = model(&messages).map_err(model_error)?;
    Ok(parse_task(&reply, mode).unwrap_or_else(|_| fallback_task(topic, mode)))
}
