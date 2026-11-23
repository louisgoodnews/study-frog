//! Classroom use cases (STORY-0209, TASK-0290): start a session from vault material, advance the
//! lecture by one segment, have a simulated student ask a question, and handle a message from the
//! user. Each turn is written to the store as soon as it exists, before the next one is generated.
//! Every use case checks provider consent before any vault text is sent.

use studyfrog_agent::lecture_prompt::{TeacherMove, teacher_messages};
use studyfrog_agent::lecture_student::student_messages;
use studyfrog_core::classroom::{ClassroomSession, Participant, SourceRef};
use studyfrog_core::classroom_ops::{add_source, append_turn, new_session, participant_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::compose_ulid;
use studyfrog_store::classroom_store::save_session;

use crate::classroom_context::{
    Lecture, classroom_error, excerpts_from, fit, is_kind, lines_of, load, model_error, persist,
    persona_of, segment_of, teacher_index,
};
use crate::classroom_material::lecture_material;
use crate::classroom_model::{Model, check_consent};
use crate::create::IdSource;
use crate::session::{ServiceError, VaultSession};

/// Creates a session grounded in `sources` (stacks, notes or lecture PDFs) and stores it. The id and
/// the creation time come from `ids`. Refused when the sources hold no text to teach from, or when
/// the participants have no teacher. No model is called.
pub fn start_session(
    vault: &VaultSession,
    ids: IdSource,
    title: &str,
    sources: &[SourceRef],
    participants: Vec<Participant>,
) -> Result<Uid, ServiceError> {
    if lecture_material(vault, sources)?.is_empty() {
        return Err(ServiceError::Invalid(
            "the chosen sources hold no text to teach from".to_owned(),
        ));
    }
    let id = compose_ulid(ids.now_ms, ids.random);
    let created = i64::try_from(ids.now_ms).unwrap_or(i64::MAX);
    let mut session =
        new_session(id, title, UnixMs(created), participants).map_err(classroom_error)?;
    for source in sources {
        add_source(&mut session, *source).map_err(classroom_error)?;
    }
    save_session(&vault.conn, &session).map_err(ServiceError::Store)?;
    Ok(id)
}

/// The teacher explains the next lecture segment, building on the conversation so far. The reply is
/// stored as a teacher turn and returned.
pub fn advance_segment(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    at: UnixMs,
) -> Result<String, ServiceError> {
    teacher_reply(lecture, model, at, TeacherMove::NextSegment)
}

/// A simulated student asks one question about the material, and the teacher answers it. Both turns
/// are stored. Returns the question and the answer.
pub fn simulated_question(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    at: UnixMs,
    student: usize,
) -> Result<(String, String), ServiceError> {
    let question = student_question(lecture, model, at, student)?;
    let answer = teacher_reply(lecture, model, at, TeacherMove::AnswerQuestion)?;
    Ok((question, answer))
}

/// The user says something (a question or an interjection). The user's turn is stored first, then the
/// teacher answers it from the material. Returns the teacher's answer.
pub fn user_message(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    at: UnixMs,
    user: usize,
    text: &str,
) -> Result<String, ServiceError> {
    check_consent(lecture.provider)?;
    let mut session = load(lecture)?;
    if !is_kind(&session, user, |p| matches!(p, Participant::User { .. })) {
        return Err(ServiceError::Invalid(
            "only the user can send a message".to_owned(),
        ));
    }
    store_turn(lecture, &mut session, user, at, text)?;
    teacher_reply(lecture, model, at, TeacherMove::AnswerQuestion)
}

/// Asks the teacher for one turn of `step`, stores it and returns it.
fn teacher_reply(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    at: UnixMs,
    step: TeacherMove,
) -> Result<String, ServiceError> {
    check_consent(lecture.provider)?;
    let mut session = load(lecture)?;
    let teacher = teacher_index(&session)
        .ok_or_else(|| classroom_error(studyfrog_core::classroom::ClassroomError::NoTeacher))?;
    let reply = ask_teacher(lecture, &session, model, step)?;
    store_turn(lecture, &mut session, teacher, at, &reply)?;
    Ok(reply)
}

/// Builds the teacher's prompt from the current segment and the recent lines, and returns the reply.
fn ask_teacher(
    lecture: &Lecture<'_>,
    session: &ClassroomSession,
    model: &mut Model<'_>,
    step: TeacherMove,
) -> Result<String, ServiceError> {
    let material = lecture_material(lecture.vault, &session.sources)?;
    let excerpts = excerpts_from(&material, segment_of(session));
    let lines = lines_of(session);
    let packed = fit(&excerpts, &lines);
    let teacher = teacher_index(session).and_then(|i| session.participants.get(i));
    let name = teacher.map_or("the teacher", participant_name);
    let messages = teacher_messages(name, &packed.excerpts, packed.summary, &packed.turns, step);
    model(&messages).map_err(model_error)
}

/// A simulated student asks one question. The question is stored as a turn of that student and returned.
fn student_question(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    at: UnixMs,
    student: usize,
) -> Result<String, ServiceError> {
    check_consent(lecture.provider)?;
    let mut session = load(lecture)?;
    let question = ask_student(lecture, &session, model, student)?;
    store_turn(lecture, &mut session, student, at, &question)?;
    Ok(question)
}

/// Builds the student's prompt, with the questions they already asked, and returns the question.
fn ask_student(
    lecture: &Lecture<'_>,
    session: &ClassroomSession,
    model: &mut Model<'_>,
    student: usize,
) -> Result<String, ServiceError> {
    let persona = persona_of(session, student)?;
    let asked: Vec<&str> = session
        .turns
        .iter()
        .filter(|t| t.speaker == student)
        .map(|t| t.text.as_str())
        .collect();
    let material = lecture_material(lecture.vault, &session.sources)?;
    let excerpts = excerpts_from(&material, segment_of(session));
    let lines = lines_of(session);
    let packed = fit(&excerpts, &lines);
    let messages = student_messages(
        &persona,
        &packed.excerpts,
        packed.summary,
        &packed.turns,
        &asked,
    );
    model(&messages).map_err(model_error)
}

/// Appends a turn to the in-memory session and writes the session to the store.
fn store_turn(
    lecture: &Lecture<'_>,
    session: &mut ClassroomSession,
    speaker: usize,
    at: UnixMs,
    text: &str,
) -> Result<(), ServiceError> {
    append_turn(session, speaker, at, text.trim()).map_err(classroom_error)?;
    persist(lecture, session)
}
