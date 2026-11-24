#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Classroom screen (TASK-0293): the session form, the role-badged lecture chat and its controls.
//! Model turns run through the real worker and service, with a fake transport and no network.
use classroom_common::{asked, chat_harness, classroom, connect, french, started, teacher_reply};
use common::flow::{opened, send};
use egui_kittest::kittest::Queryable;
use studyfrog_core::classroom::{SourceKind, SourceRef};
use studyfrog_service::classroom_task::Assignees;
use studyfrog_ui::classroom_types::{ClassroomEffect, ClassroomMsg};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::msg::Msg;

mod classroom_common;
mod common;

#[test]
fn the_form_refuses_a_start_without_a_source_or_a_model() {
    let (_vault, mut m, _w) = opened("classroom-refuse");
    let fx = classroom(&mut m, ClassroomMsg::Start);
    assert!(fx.is_empty(), "nothing is started without a model");
    assert_eq!(
        m.state.classroom.error.as_deref(),
        Some("Choose at least one stack or PDF to teach from.")
    );

    let uid = french(&m);
    classroom(
        &mut m,
        ClassroomMsg::ToggleSource(SourceRef {
            kind: SourceKind::Stack,
            uid,
        }),
    );
    let fx = classroom(&mut m, ClassroomMsg::Start);
    assert!(fx.is_empty());
    assert_eq!(
        m.state.classroom.error.as_deref(),
        Some("Connect a model in Assistant first.")
    );
}

#[test]
fn start_asks_for_a_session_with_the_chosen_sources_provider_and_personas() {
    let (_vault, mut m, _w) = opened("classroom-start-effect");
    connect(&mut m);
    let uid = french(&m);
    classroom(
        &mut m,
        ClassroomMsg::ToggleSource(SourceRef {
            kind: SourceKind::Stack,
            uid,
        }),
    );
    classroom(&mut m, ClassroomMsg::Students(3));
    let fx = classroom(&mut m, ClassroomMsg::Start);
    let [
        Effect::Classroom(ClassroomEffect::Start {
            title,
            sources,
            participants,
            provider,
        }),
    ] = fx.as_slice()
    else {
        panic!("one start effect, got {fx:?}");
    };
    assert_eq!(title, "Classroom");
    assert_eq!(sources.len(), 1);
    assert_eq!(provider.name, "LocalLLM");
    let names: Vec<&str> = participants
        .iter()
        .map(studyfrog_core::classroom_ops::participant_name)
        .collect();
    assert_eq!(names, ["Teacher", "You", "Ada", "Ben", "Mia"]);
    assert!(m.state.classroom.busy, "the start is waiting on the worker");
}

#[test]
fn starting_stores_the_session_through_the_worker() {
    let (_vault, m, _w) = started();
    let s = m.state.classroom.session.as_ref().unwrap();
    assert_eq!(s.sources.len(), 1);
    assert_eq!(s.participants.len(), 4, "teacher, you and two students");
    assert!(!m.state.classroom.busy);
    assert!(m.state.classroom.error.is_none());
}

#[test]
fn the_chat_shows_a_distinct_role_badge_for_teacher_students_and_user() {
    let (_vault, m, _w) = started();
    let h = chat_harness(m);
    for badge in ["Teacher role", "Student role", "User role"] {
        assert!(
            h.query_by_label(badge).is_some(),
            "the {badge} badge is drawn"
        );
    }
    assert!(h.query_by_label("Welcome to the lecture.").is_some());
    assert!(h.query_by_label("What is a noun?").is_some());
    assert!(h.query_by_label("Why does it matter?").is_some());
}

#[test]
fn continue_lecture_is_a_worker_effect_that_stores_the_teacher_turn() {
    let (_vault, mut m, mut w) = started();
    let cfg = common::temp_dir("classroom-cfg");
    w.config_dir = Some(cfg.path.clone());
    w.transport = Some(teacher_reply);
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::Advance));
    let s = m.state.classroom.session.as_ref().unwrap();
    assert_eq!(s.turns.len(), 1, "the teacher turn is stored");
    assert_eq!(s.turns[0].text, "Plants turn light into sugar.");
    assert!(!m.state.classroom.busy);
    assert!(m.state.classroom.error.is_none());
}

#[test]
fn a_build_without_a_network_reports_the_failure_and_stores_nothing() {
    let (_vault, mut m, mut w) = started();
    w.transport = None;
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::Advance));
    let s = m.state.classroom.session.as_ref().unwrap();
    assert!(s.turns.is_empty());
    assert!(!m.state.classroom.busy);
    assert_eq!(
        m.state.classroom.error.as_deref(),
        Some("This build cannot reach the network.")
    );
}

#[test]
fn the_lecture_controls_ask_the_right_effect_for_each_button() {
    let (_vault, m, _w) = started();
    let mut h = chat_harness(m);

    h.get_by_label("Continue lecture").click();
    h.run_steps(3);
    assert!(asked(&h, |e| matches!(
        e,
        Effect::Classroom(ClassroomEffect::Advance { .. })
    )));

    h.state_mut().0.state.classroom.busy = false;
    h.get_by_label("Ask Ada").click();
    h.run_steps(3);
    assert!(asked(&h, |e| matches!(
        e,
        Effect::Classroom(ClassroomEffect::AskStudent { student: 2, .. })
    )));

    h.state_mut().0.state.classroom.busy = false;
    h.get_by_label("Assign task").click();
    h.run_steps(3);
    assert!(asked(&h, |e| matches!(
        e,
        Effect::Classroom(ClassroomEffect::AssignTask {
            assignees: Assignees::Solo(1),
            ..
        })
    )));
}
