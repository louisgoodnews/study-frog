//! The pieces a classroom turn is built from (STORY-0209, TASK-0290): loading and saving a session,
//! the lecture window the context budgeter lets through, and the persona of a simulated student.
//! Each saved turn is written to the store before the next one is generated.

use studyfrog_agent::lecture_budget::{chars_for_tokens, pack};
use studyfrog_agent::lecture_prompt::Line;
use studyfrog_agent::lecture_student::{PRESET_PERSONAS, Persona};
use studyfrog_core::classroom::{ClassroomError, ClassroomSession, Participant};
use studyfrog_core::classroom_ops::participant_name;
use studyfrog_core::uid::Uid;
use studyfrog_store::classroom_store::{load_session, update_session};

use crate::session::{ServiceError, VaultSession};

/// Tokens of lecture context a turn may send: the material from the current segment on, and the
/// recent conversation. Small local models degrade past this, so it stays modest.
pub const LECTURE_BUDGET_TOKENS: usize = 1_500;

/// The lecture being run: the vault it reads from, the provider that answers, and the session.
#[derive(Clone, Copy)]
pub struct Lecture<'a> {
    /// The open vault: its index holds the sources and the session rows.
    pub vault: &'a VaultSession,
    /// The model connection the turns are sent to.
    pub provider: &'a studyfrog_agent::provider::Provider,
    /// The session the turns are added to.
    pub id: Uid,
}

/// The stored session, or an error when no session has the id.
pub fn load(lecture: &Lecture<'_>) -> Result<ClassroomSession, ServiceError> {
    load_session(&lecture.vault.conn, lecture.id)
        .map_err(ServiceError::Store)?
        .ok_or_else(|| ServiceError::Invalid("no classroom session has that id".to_owned()))
}

/// Writes the whole session back. Refused when the session is no longer stored.
pub fn persist(lecture: &Lecture<'_>, session: &ClassroomSession) -> Result<(), ServiceError> {
    if update_session(&lecture.vault.conn, session).map_err(ServiceError::Store)? {
        Ok(())
    } else {
        Err(ServiceError::Invalid(
            "no classroom session has that id".to_owned(),
        ))
    }
}

/// Maps a refused classroom change to a message the user can read.
pub fn classroom_error(e: ClassroomError) -> ServiceError {
    let text = match e {
        ClassroomError::EmptyText => "the message is empty".to_owned(),
        ClassroomError::UnknownParticipant(_) => {
            "that participant is not in the session".to_owned()
        }
        ClassroomError::NoTeacher => "a session needs a teacher".to_owned(),
        ClassroomError::TurnOutOfOrder => "the turn is earlier than the last one".to_owned(),
        other => format!("the session refused the change: {other:?}"),
    };
    ServiceError::Invalid(text)
}

/// A model that cannot be reached or refuses is reported as an I/O failure.
pub fn model_error(e: String) -> ServiceError {
    ServiceError::Io(e)
}

/// The index of the first teacher, if any.
pub fn teacher_index(session: &ClassroomSession) -> Option<usize> {
    session
        .participants
        .iter()
        .position(|p| matches!(p, Participant::Teacher { .. }))
}

/// Whether the participant at `index` is the given kind (checked by the caller's predicate).
pub fn is_kind(session: &ClassroomSession, index: usize, kind: fn(&Participant) -> bool) -> bool {
    session.participants.get(index).is_some_and(kind)
}

/// The number of teacher turns so far: the segment of the lecture the teacher is on.
pub fn segment_of(session: &ClassroomSession) -> usize {
    session
        .turns
        .iter()
        .filter(|t| {
            is_kind(session, t.speaker, |p| {
                matches!(p, Participant::Teacher { .. })
            })
        })
        .count()
}

/// The chat as lines, oldest first, with the speakers' names.
pub fn lines_of(session: &ClassroomSession) -> Vec<Line<'_>> {
    session
        .turns
        .iter()
        .map(|t| Line {
            speaker: session
                .participants
                .get(t.speaker)
                .map_or("?", participant_name),
            text: &t.text,
        })
        .collect()
}

/// The material from segment `segment` on, in order. Past the last piece, the last piece is repeated.
pub fn excerpts_from(material: &[String], segment: usize) -> Vec<&str> {
    let start = segment.min(material.len().saturating_sub(1));
    material
        .get(start..)
        .unwrap_or(&[])
        .iter()
        .map(String::as_str)
        .collect()
}

/// The persona of the simulated student at `index`: their name from the session, and a stance from
/// the presets by their order among the students, so the first is the curious beginner. Refused when that participant is not a student.
pub fn persona_of(session: &ClassroomSession, index: usize) -> Result<Persona<'_>, ServiceError> {
    match session.participants.get(index) {
        Some(Participant::SimulatedStudent { name }) => Ok(Persona {
            name,
            stance: PRESET_PERSONAS[student_ordinal(session, index) % PRESET_PERSONAS.len()].stance,
        }),
        Some(_) => Err(ServiceError::Invalid(
            "only a simulated student can ask a question".to_owned(),
        )),
        None => Err(classroom_error(ClassroomError::UnknownParticipant(index))),
    }
}

/// How many simulated students come before the participant at `index`: 0 for the first student.
fn student_ordinal(session: &ClassroomSession, index: usize) -> usize {
    session.participants[..index]
        .iter()
        .filter(|p| matches!(p, Participant::SimulatedStudent { .. }))
        .count()
}

/// The budget for one turn, in characters.
pub fn budget_chars() -> usize {
    chars_for_tokens(LECTURE_BUDGET_TOKENS)
}

/// The packed context of one turn: the excerpts and recent lines that fit the budget. `pack` is given
/// the summary as empty because the session does not keep a rolling summary yet.
pub fn fit<'a>(
    excerpts: &[&'a str],
    lines: &[Line<'a>],
) -> studyfrog_agent::lecture_budget::Packed<'a> {
    pack(budget_chars(), "", excerpts, lines)
}
