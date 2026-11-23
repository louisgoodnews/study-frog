//! Pure transitions for a classroom session: creation and chat turns. Each function either
//! applies its change or returns an error and leaves the session as it was.

use crate::classroom::{ChatTurn, ClassroomError, ClassroomSession, Participant, SourceRef};
use crate::time::UnixMs;
use crate::uid::Uid;

/// Creates a session with its participants. There must be a teacher, and the title and every
/// name must be non-blank.
pub fn new_session(
    id: Uid,
    title: &str,
    created_at: UnixMs,
    participants: Vec<Participant>,
) -> Result<ClassroomSession, ClassroomError> {
    if is_blank(title) {
        return Err(ClassroomError::EmptyText);
    }
    if participants.iter().any(|p| is_blank(participant_name(p))) {
        return Err(ClassroomError::EmptyText);
    }
    if !participants
        .iter()
        .any(|p| matches!(p, Participant::Teacher { .. }))
    {
        return Err(ClassroomError::NoTeacher);
    }
    Ok(ClassroomSession {
        id,
        title: title.to_owned(),
        created_at,
        participants,
        turns: Vec::new(),
        assignments: Vec::new(),
        submissions: Vec::new(),
        sources: Vec::new(),
    })
}

/// Appends a chat turn by a participant. Turns must not go back in time, and the text must not
/// be blank.
pub fn append_turn(
    session: &mut ClassroomSession,
    speaker: usize,
    at: UnixMs,
    text: &str,
) -> Result<(), ClassroomError> {
    check_participant(session, speaker)?;
    if is_blank(text) {
        return Err(ClassroomError::EmptyText);
    }
    if session.turns.last().is_some_and(|last| at < last.at) {
        return Err(ClassroomError::TurnOutOfOrder);
    }
    let seq = u32::try_from(session.turns.len()).unwrap_or(u32::MAX);
    session.turns.push(ChatTurn {
        seq,
        speaker,
        at,
        text: text.to_owned(),
    });
    Ok(())
}

/// Adds a vault item as a source of the lecture. Each item is listed once.
pub fn add_source(session: &mut ClassroomSession, source: SourceRef) -> Result<(), ClassroomError> {
    if session.sources.contains(&source) {
        return Err(ClassroomError::DuplicateSource);
    }
    session.sources.push(source);
    Ok(())
}

/// Checks that a participant index names someone in the session.
pub fn check_participant(session: &ClassroomSession, index: usize) -> Result<(), ClassroomError> {
    if index < session.participants.len() {
        Ok(())
    } else {
        Err(ClassroomError::UnknownParticipant(index))
    }
}

/// Whether a participant can be assigned work: a simulated student or the user, not the teacher.
pub fn is_student(participant: &Participant) -> bool {
    !matches!(participant, Participant::Teacher { .. })
}

/// The display name of a participant.
pub fn participant_name(participant: &Participant) -> &str {
    match participant {
        Participant::Teacher { name }
        | Participant::SimulatedStudent { name }
        | Participant::User { name } => name,
    }
}

/// Whether text is empty or only whitespace.
pub fn is_blank(text: &str) -> bool {
    text.trim().is_empty()
}
