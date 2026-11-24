#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Classroom screen (TASK-0293): the session form, the role-badged lecture chat and its controls.
//! Model turns run through the real worker and service, with a fake transport and no network.
use classroom_common::{asked, chat_harness, classroom, connect, sized, started};
use common::flow::opened;
use eframe::egui::Rect;
use egui_kittest::kittest::Queryable;
use studyfrog_ui::classroom_types::{ClassroomEffect, ClassroomMsg};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

mod classroom_common;
mod common;

#[test]
fn the_composer_sends_the_user_message_asynchronously() {
    let (_vault, m, _w) = started();
    let mut h = chat_harness(m);
    apply(
        &mut h.state_mut().0,
        Msg::Classroom(ClassroomMsg::Input("Is a verb a noun?".into())),
    );
    h.run_steps(2);
    h.get_by_label("Send").click();
    h.run_steps(3);
    assert!(asked(&h, |e| matches!(
        e,
        Effect::Classroom(ClassroomEffect::Say { user: 1, text, .. }) if text == "Is a verb a noun?"
    )));
    assert_eq!(
        h.state().0.state.classroom.input,
        "",
        "the composer is cleared"
    );
}

#[test]
fn a_blank_message_is_not_sent() {
    let (_vault, mut m, _w) = started();
    classroom(&mut m, ClassroomMsg::Input("   ".into()));
    let fx = classroom(&mut m, ClassroomMsg::Say);
    assert!(fx.is_empty());
    assert_eq!(
        m.state.classroom.error.as_deref(),
        Some("Type a question or comment first.")
    );
}

#[test]
fn the_setup_form_starts_a_session_from_the_screen() {
    let (_vault, mut m, _w) = opened("classroom-form-screen");
    connect(&mut m);
    m.state.screen = Screen::Classroom;
    let mut h = common::views::harness(m);
    h.run_steps(4);
    assert!(h.query_by_label("Session title").is_some());
    assert!(h.query_all_by_label("Simulated students").next().is_some());
    assert!(
        h.query_by_label("French").is_some(),
        "the stack can be ticked"
    );

    h.get_by_label("French").click();
    h.run_steps(2);
    h.get_by_label("Start session").click();
    h.run_steps(3);
    assert!(asked(&h, |e| matches!(
        e,
        Effect::Classroom(ClassroomEffect::Start { .. })
    )));
}

#[test]
fn back_to_setup_leaves_the_session_on_screen() {
    let (_vault, m, _w) = started();
    let mut h = chat_harness(m);
    h.get_by_label("Back to setup").click();
    h.run_steps(3);
    assert!(h.state().0.state.classroom.session.is_none());
}

#[test]
fn the_setup_and_chat_fit_the_window_at_960_by_640_and_1280_by_800() {
    let inside = |r: Rect, w: f32, h: f32| r.min.x >= 0.0 && r.max.x <= w && r.max.y <= h;
    for (w, h) in [(960.0, 640.0), (1280.0, 800.0)] {
        let (_vault, mut m, _work) = opened("classroom-layout");
        connect(&mut m);
        m.state.screen = Screen::Classroom;
        let harness = sized(m, w, h);
        let start = harness.get_by_label("Start session").rect();
        assert!(inside(start, w, h), "Start session {start:?} fits {w}x{h}");

        let (_vault, mut m, _work) = started();
        m.state.screen = Screen::Classroom;
        let harness = sized(m, w, h);
        let send_button = harness.get_by_label("Send").rect();
        assert!(
            inside(send_button, w, h),
            "Send {send_button:?} fits {w}x{h}"
        );
    }
}
