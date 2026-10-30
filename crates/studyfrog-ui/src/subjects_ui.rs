//! The teachers and subjects of the vault, edited on the settings screen. Pure.

use studyfrog_config::subjects::{Subject, SubjectBook};

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// The book and the drafts of the two "add" forms.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubjectsState {
    /// What is stored.
    pub book: SubjectBook,
    /// Why the stored file is not used, if so.
    pub error: Option<String>,
    /// The name typed for a new teacher.
    pub teacher_draft: String,
    /// The name typed for a new subject.
    pub subject_draft: String,
    /// The teacher chosen for the new subject.
    pub subject_teacher: Option<String>,
}

/// Messages of the subjects section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectsMsg {
    /// The stored book was read, or a save finished; carries the book and a reason it was not used.
    Loaded(Result<(SubjectBook, Option<String>), String>),
    /// The new teacher's name changed.
    TeacherDraft(String),
    /// Add the typed teacher.
    AddTeacher,
    /// Remove a teacher (their subjects keep existing without one).
    RemoveTeacher(String),
    /// The new subject's name changed.
    SubjectDraft(String),
    /// The teacher of the new subject changed.
    SubjectTeacher(Option<String>),
    /// Add the typed subject.
    AddSubject,
    /// Remove a subject.
    RemoveSubject(String),
    /// Give a subject another teacher.
    SetTeacher(String, Option<String>),
}

fn failed(state: State, text: String) -> (State, Vec<Effect>) {
    let notice = Some(Notice {
        kind: NoticeKind::Error,
        text,
    });
    (State { notice, ..state }, Vec::new())
}

/// Store a changed book.
fn changed(state: State, book: SubjectBook, keep: SubjectsState) -> (State, Vec<Effect>) {
    let subjects = SubjectsState { ..keep };
    (
        State { subjects, ..state },
        vec![Effect::SaveSubjects(book)],
    )
}

/// Handle the messages of the subjects section.
pub fn update(state: State, msg: SubjectsMsg) -> (State, Vec<Effect>) {
    let s = state.subjects.clone();
    let mut book = s.book.clone();
    match msg {
        SubjectsMsg::Loaded(Ok((book, error))) => {
            let subjects = SubjectsState { book, error, ..s };
            (State { subjects, ..state }, Vec::new())
        }
        SubjectsMsg::Loaded(Err(e)) => failed(state, format!("Could not save the subjects: {e}")),
        SubjectsMsg::TeacherDraft(t) => draft(
            state,
            SubjectsState {
                teacher_draft: t,
                ..s
            },
        ),
        SubjectsMsg::SubjectDraft(t) => draft(
            state,
            SubjectsState {
                subject_draft: t,
                ..s
            },
        ),
        SubjectsMsg::SubjectTeacher(t) => draft(
            state,
            SubjectsState {
                subject_teacher: t,
                ..s
            },
        ),
        SubjectsMsg::AddTeacher => {
            let name = s.teacher_draft.trim().to_owned();
            if name.is_empty() {
                return failed(state, "Type the teacher's name first.".to_owned());
            }
            book.teachers.push(name);
            changed(
                state,
                book,
                SubjectsState {
                    teacher_draft: String::new(),
                    ..s
                },
            )
        }
        SubjectsMsg::RemoveTeacher(name) => {
            book.teachers.retain(|t| *t != name);
            changed(state, book, s)
        }
        SubjectsMsg::AddSubject => {
            let name = s.subject_draft.trim().to_owned();
            if name.is_empty() {
                return failed(state, "Type the subject's name first.".to_owned());
            }
            book.subjects.push(Subject {
                name,
                teacher: s.subject_teacher.clone(),
            });
            changed(
                state,
                book,
                SubjectsState {
                    subject_draft: String::new(),
                    ..s
                },
            )
        }
        SubjectsMsg::RemoveSubject(name) => {
            book.subjects.retain(|x| x.name != name);
            changed(state, book, s)
        }
        SubjectsMsg::SetTeacher(name, teacher) => {
            if let Some(x) = book.subjects.iter_mut().find(|x| x.name == name) {
                x.teacher = teacher;
            }
            changed(state, book, s)
        }
    }
}

fn draft(state: State, subjects: SubjectsState) -> (State, Vec<Effect>) {
    (State { subjects, ..state }, Vec::new())
}
