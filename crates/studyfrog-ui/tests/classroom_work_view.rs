#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Classroom assignments, part 2 (EPIC-068, TASK-0294): the drawer's controls and effects, the graded
//! view with score, grade, rubric breakdown and feedback, and the layout at phone and desktop widths.
use classroom_common::{
    chat_harness, classroom, group_task, online, open, sized, solo_task, started,
};
use common::flow::send;
use egui_kittest::kittest::Queryable;
use studyfrog_agent::lecture_grade::label_of;
use studyfrog_core::classroom::Owner;
use studyfrog_core::classroom_status::grade_for;
use studyfrog_ui::classroom_types::{ClassroomEffect, ClassroomMsg};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

mod classroom_common;
mod common;

#[test]
fn a_graded_task_shows_the_score_grade_rubric_breakdown_and_feedback() {
    let (_vault, mut m, mut w) = started();
    let _cfg = online(&mut w);
    let solo = solo_task(&mut m, &mut w);
    open(&mut m, &mut w, solo);
    classroom(
        &mut m,
        ClassroomMsg::DraftEdit("Plants make sugar from light.".into()),
    );
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::TurnIn));
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::AskGrade));
    assert!(
        m.state.classroom.error.is_none(),
        "{:?}",
        m.state.classroom.error
    );

    // The grade is read back from the store after the worker, so the breakdown survived storage.
    let s = m.state.classroom.session.as_ref().unwrap();
    let grade = grade_for(s, solo, Owner::Participant(1))
        .expect("graded")
        .clone();
    assert!(!grade.criteria.is_empty(), "one mark per rubric point");
    let letter = label_of(grade.points * 100 / grade.max);
    let h = chat_harness(m);
    assert!(h.query_by_label("Teacher's grade").is_some());
    assert!(
        h.query_by_label(&format!("Score: {} of {}", grade.points, grade.max))
            .is_some()
    );
    assert!(h.query_by_label(&format!("Grade: {letter}")).is_some());
    for mark in &grade.criteria {
        assert!(
            h.query_by_label(&mark.criterion).is_some(),
            "rubric point shown"
        );
        let verdict = match (mark.checked, mark.met) {
            (false, _) => "Rubric point Not checked",
            (true, true) => "Rubric point Met",
            (true, false) => "Rubric point Missing",
        };
        assert!(h.query_all_by_label(verdict).next().is_some(), "{verdict}");
    }
    if !grade.feedback.trim().is_empty() {
        assert!(h.query_by_label("Feedback").is_some());
    }
}

#[test]
fn the_group_composer_and_turn_in_send_their_effects_from_the_drawer() {
    let (_vault, mut m, mut w) = started();
    let _cfg = online(&mut w);
    let group = group_task(&mut m, &mut w);
    open(&mut m, &mut w, group);
    let mut h = chat_harness(m);
    apply(
        &mut h.state_mut().0,
        Msg::Classroom(ClassroomMsg::GroupInput("Hi team".into())),
    );
    h.run_steps(2);
    h.get_by_label("Send to group").click();
    h.run_steps(3);
    assert!(h.state().1.iter().any(|e| matches!(
        e,
        Effect::Classroom(ClassroomEffect::GroupMessage { group: 0, text, .. })
            if text == "Hi team"
    )));
    assert!(h.query_by_label("Message to your group").is_some());
    assert!(h.query_by_label("Your answer").is_some());
}

#[test]
fn the_drawer_turn_in_button_asks_for_a_turn_in_of_the_drafted_answer() {
    let (_vault, mut m, mut w) = started();
    let _cfg = online(&mut w);
    let solo = solo_task(&mut m, &mut w);
    open(&mut m, &mut w, solo);
    let mut h = chat_harness(m);
    apply(
        &mut h.state_mut().0,
        Msg::Classroom(ClassroomMsg::DraftEdit("My answer".into())),
    );
    h.run_steps(2);
    h.get_by_label("Turn in").click();
    h.run_steps(3);
    assert!(h.state().1.iter().any(|e| matches!(
        e,
        Effect::Classroom(ClassroomEffect::TurnIn { submitter: 1, text, .. }) if text == "My answer"
    )));
}

#[test]
fn the_assignments_panel_fits_phone_and_desktop_widths_without_horizontal_overflow() {
    for (w_px, h_px) in [(360.0, 640.0), (960.0, 640.0), (1280.0, 800.0)] {
        let (_vault, mut m, mut w) = started();
        let _cfg = online(&mut w);
        group_task(&mut m, &mut w);
        m.state.screen = Screen::Classroom;
        let harness = sized(m, w_px, h_px);
        for label in ["Send", "Assign group task"] {
            let r = harness.get_by_label(label).rect();
            assert!(
                r.min.x >= 0.0 && r.max.x <= w_px,
                "{label} {r:?} fits the {w_px}-pixel width"
            );
        }
    }
}
