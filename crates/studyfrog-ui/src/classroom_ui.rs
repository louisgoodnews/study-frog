//! The Classroom screen's reducer (TASK-0293): pure updates to the state and the effects they ask for.
//! Every turn is one effect; while one runs, the controls are held back (`busy`).

use studyfrog_agent::lecture_student::PRESET_PERSONAS;
use studyfrog_agent::provider::Provider;
use studyfrog_core::classroom::{ClassroomSession, Participant, SourceKind, SourceRef};
use studyfrog_core::classroom_ops::is_blank;
use studyfrog_core::uid::Uid;
use studyfrog_service::classroom_task::Assignees;

pub mod saved;
use crate::classroom_types::{ClassroomEffect, ClassroomMsg};
use crate::effect::Effect;
use crate::state::State;

/// The most simulated students a session takes: the built-in presets.
pub const MAX_STUDENTS: usize = PRESET_PERSONAS.len();

/// Advance the Classroom screen by one message and return the effects to run.
pub fn update(mut state: State, msg: ClassroomMsg) -> (State, Vec<Effect>) {
    let fx = apply(&mut state, msg);
    (state, fx)
}

fn apply(state: &mut State, msg: ClassroomMsg) -> Vec<Effect> {
    match msg {
        ClassroomMsg::Title(title) => {
            state.classroom.title = title;
            Vec::new()
        }
        ClassroomMsg::ToggleSource(source) => {
            toggle_source(&mut state.classroom.sources, source);
            Vec::new()
        }
        ClassroomMsg::Students(n) => {
            state.classroom.students = n.min(MAX_STUDENTS);
            Vec::new()
        }
        ClassroomMsg::ChooseProvider(name) => {
            state.classroom.provider = Some(name);
            Vec::new()
        }
        ClassroomMsg::Input(text) => {
            state.classroom.input = text;
            Vec::new()
        }
        ClassroomMsg::Start => start(state),
        ClassroomMsg::Advance => turn(state, |s, provider| {
            Ok(ClassroomEffect::Advance { id: s.id, provider })
        }),
        ClassroomMsg::AskStudent(student) => turn(state, |s, provider| {
            check_student(s, student)?;
            Ok(ClassroomEffect::AskStudent {
                id: s.id,
                provider,
                student,
            })
        }),
        ClassroomMsg::Say => say(state),
        ClassroomMsg::AssignTask => turn(state, |s, provider| {
            let user = user_index(s).ok_or("This session has no user to set a task for.")?;
            Ok(ClassroomEffect::AssignTask {
                id: s.id,
                provider,
                assignees: Assignees::Solo(user),
            })
        }),
        ClassroomMsg::OpenTask(_)
        | ClassroomMsg::DraftEdit(_)
        | ClassroomMsg::GroupInput(_)
        | ClassroomMsg::SendGroup
        | ClassroomMsg::TurnIn
        | ClassroomMsg::AskGrade
        | ClassroomMsg::AssignGroups => crate::classroom_work_ui::apply(state, msg),
        ClassroomMsg::Saved(msg) => saved::apply(state, msg),
        ClassroomMsg::Leave => {
            if !state.classroom.busy {
                state.classroom.session = None;
                state.classroom.error = None;
            }
            saved::refresh()
        }
        ClassroomMsg::Done { session, error } => {
            state.classroom.busy = false;
            state.classroom.session = session.or_else(|| state.classroom.session.take());
            state.classroom.error = error;
            saved::refresh()
        }
    }
}

/// Adds the source when it is not chosen, removes it when it is.
fn toggle_source(sources: &mut Vec<SourceRef>, source: SourceRef) {
    match sources.iter().position(|s| *s == source) {
        Some(at) => {
            sources.remove(at);
        }
        None => sources.push(source),
    }
}

/// Creates a session from the form. Refused with a message when no source or no model is chosen.
fn start(state: &mut State) -> Vec<Effect> {
    if state.classroom.busy {
        return Vec::new();
    }
    let outcome = start_effect(state);
    finish_request(state, outcome)
}

fn start_effect(state: &State) -> Result<ClassroomEffect, String> {
    if state.classroom.sources.is_empty() {
        return Err("Choose at least one stack or PDF to teach from.".to_owned());
    }
    let provider = chosen_provider(state)?;
    let title = state.classroom.title.trim();
    let title = if title.is_empty() { "Classroom" } else { title };
    Ok(ClassroomEffect::Start {
        title: title.to_owned(),
        sources: state.classroom.sources.clone(),
        participants: participants(state.classroom.students),
        provider,
    })
}

/// The teacher, the user and `students` simulated students, in that order.
pub fn participants(students: usize) -> Vec<Participant> {
    let mut out = vec![
        Participant::Teacher {
            name: "Teacher".to_owned(),
        },
        Participant::User {
            name: "You".to_owned(),
        },
    ];
    out.extend(
        PRESET_PERSONAS
            .iter()
            .take(students.min(MAX_STUDENTS))
            .map(|p| Participant::SimulatedStudent {
                name: p.name.to_owned(),
            }),
    );
    out
}

/// Sends the composer text as the user. Blank text is refused before anything is queued.
fn say(state: &mut State) -> Vec<Effect> {
    if state.classroom.busy {
        return Vec::new();
    }
    let text = state.classroom.input.trim().to_owned();
    if is_blank(&text) {
        state.classroom.error = Some("Type a question or comment first.".to_owned());
        return Vec::new();
    }
    let outcome = turn_effect(state, |s, provider| {
        let user = user_index(s).ok_or("This session has no user to speak for.")?;
        Ok(ClassroomEffect::Say {
            id: s.id,
            provider,
            user,
            text,
        })
    });
    if outcome.is_ok() {
        state.classroom.input.clear();
    }
    finish_request(state, outcome)
}

/// Runs a turn on the session on screen, unless one is already running.
pub(crate) fn turn(
    state: &mut State,
    build: impl FnOnce(&ClassroomSession, Provider) -> Result<ClassroomEffect, String>,
) -> Vec<Effect> {
    if state.classroom.busy {
        return Vec::new();
    }
    let outcome = turn_effect(state, build);
    finish_request(state, outcome)
}

pub(crate) fn turn_effect(
    state: &State,
    build: impl FnOnce(&ClassroomSession, Provider) -> Result<ClassroomEffect, String>,
) -> Result<ClassroomEffect, String> {
    let session = state
        .classroom
        .session
        .as_ref()
        .ok_or("Start a session first.")?;
    build(session, chosen_provider(state)?)
}

/// Sets the busy flag and asks for the effect when it was built, or records why it was not.
pub(crate) fn finish_request(
    state: &mut State,
    outcome: Result<ClassroomEffect, String>,
) -> Vec<Effect> {
    match outcome {
        Ok(effect) => {
            state.classroom.busy = true;
            state.classroom.error = None;
            vec![Effect::Classroom(effect)]
        }
        Err(reason) => {
            state.classroom.error = Some(reason);
            Vec::new()
        }
    }
}

/// The model the screen uses: the one chosen here, else the Assistant's selection, else the first.
pub fn chosen_provider(state: &State) -> Result<Provider, String> {
    let providers = &state.agent.connections.providers;
    let name = state
        .classroom
        .provider
        .as_ref()
        .or(state.agent.selected.as_ref());
    let found = match name {
        Some(name) => providers.iter().find(|p| &p.name == name),
        None => providers.first(),
    };
    found
        .or_else(|| providers.first())
        .cloned()
        .ok_or_else(|| "Connect a model in Assistant first.".to_owned())
}

fn check_student(session: &ClassroomSession, index: usize) -> Result<(), String> {
    match session.participants.get(index) {
        Some(Participant::SimulatedStudent { .. }) => Ok(()),
        _ => Err("Only a simulated student can ask a question.".to_owned()),
    }
}

/// The participant index of the user, if the session has one.
pub fn user_index(session: &ClassroomSession) -> Option<usize> {
    session
        .participants
        .iter()
        .position(|p| matches!(p, Participant::User { .. }))
}

/// Whether a source of this kind is in the list, for the form's checkboxes.
pub fn is_chosen(sources: &[SourceRef], kind: SourceKind, uid: Uid) -> bool {
    sources.iter().any(|s| s.kind == kind && s.uid == uid)
}
