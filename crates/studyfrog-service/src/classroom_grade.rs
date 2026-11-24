//! Teacher grading of a turned-in answer (STORY-0210, TASK-0292). The teacher judges the answer against the
//! task's rubric through the model; when the model call fails, its reply does not read, or the provider may
//! not receive vault text, the deterministic keyword check grades the answer instead. The grade is recorded
//! once, through the core transition, and the session is written in one go.

use studyfrog_agent::lecture_grade::{
    GRADE_MAX, Grade, grade_answer, grade_messages, keyword_grade,
};
use studyfrog_agent::provider::may_send;
use studyfrog_core::classroom::{
    Assignment, ClassroomError, ClassroomSession, GradeReport, Owner, Submission,
};
use studyfrog_core::classroom_ops::participant_name;
use studyfrog_core::classroom_work::{check_owner, record_grade};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;

use crate::classroom_context::{
    Lecture, classroom_error, find_assignment, load, persist, submission_of, teacher_index,
};
use crate::classroom_model::Model;
use crate::session::ServiceError;

/// Grades the turned-in answer of `owner` for `assignment`, records the grade and returns it. The points
/// are stored as the score out of [`GRADE_MAX`] and the summary as the feedback; the letter is derived
/// from the points. Refused, with no model call, when the owner does not fit the task's kind, the answer
/// was never turned in, or it is already graded.
pub fn grade_submission(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    at: UnixMs,
    assignment: Uid,
    owner: Owner,
) -> Result<Grade, ServiceError> {
    let mut session = load(lecture)?;
    let grade = judge_pending(lecture, model, &session, assignment, owner)?;
    let report = GradeReport {
        points: grade.points,
        max: GRADE_MAX,
        feedback: grade.summary.clone(),
        graded_at: at,
    };
    record_grade(&mut session, assignment, owner, report).map_err(classroom_error)?;
    persist(lecture, &session)?;
    Ok(grade)
}

/// Checks the owner, the turn-in and the grade state, then grades the answer. Nothing is changed here.
fn judge_pending(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    session: &ClassroomSession,
    assignment: Uid,
    owner: Owner,
) -> Result<Grade, ServiceError> {
    let task = find_assignment(session, assignment)?;
    check_owner(task, owner).map_err(classroom_error)?;
    let answer = ungraded(session, assignment, owner)?;
    let teacher = teacher_name(session);
    Ok(judge(lecture, model, teacher, task, answer))
}

/// The answer to grade: it must have been turned in and not yet graded.
fn ungraded(
    session: &ClassroomSession,
    assignment: Uid,
    owner: Owner,
) -> Result<&Submission, ServiceError> {
    let answer = submission_of(session, assignment, owner)
        .ok_or_else(|| classroom_error(ClassroomError::NotSubmitted))?;
    if answer.grade.is_some() {
        return Err(classroom_error(ClassroomError::AlreadyGraded));
    }
    Ok(answer)
}

fn teacher_name(session: &ClassroomSession) -> &str {
    teacher_index(session)
        .and_then(|i| session.participants.get(i))
        .map_or("the teacher", participant_name)
}

/// The model's grade of the answer, or the keyword grade when the model may not be asked or fails.
fn judge(
    lecture: &Lecture<'_>,
    model: &mut Model<'_>,
    teacher: &str,
    task: &Assignment,
    answer: &Submission,
) -> Grade {
    if !may_send(lecture.provider) {
        let caveat = format!(
            "Text from your vault may not be sent to {} until you allow it; the keyword check is shown.",
            lecture.provider.name
        );
        return keyword_grade(&answer.text, &task.rubric, Some(caveat));
    }
    let messages = grade_messages(
        teacher,
        &task.title,
        &task.prompt,
        &task.rubric,
        &answer.text,
    );
    let reply = model(&messages);
    grade_answer(
        reply.as_deref().map_err(String::as_str),
        &answer.text,
        &task.rubric,
    )
}
