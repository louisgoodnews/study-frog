#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Classroom screen in the control protocol (TASK-0295): the screen word reaches it, and the
//! snapshot shows the active session and the saved sessions.
use classroom_common::started;
use common::flow::send;
use studyfrog_ui::classroom_types::ClassroomMsg;
use studyfrog_ui::control_state::{screen_name, state_json};
use studyfrog_ui::control_words::screen_by_name;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

mod classroom_common;
mod common;

#[test]
fn the_classroom_screen_word_and_its_name_match() {
    assert_eq!(screen_by_name("classroom"), Some(Screen::Classroom));
    assert_eq!(screen_name(Screen::Classroom), "classroom");
}

#[test]
fn the_snapshot_shows_the_active_session_then_the_saved_ones() {
    let (_vault, mut m, mut w) = started();
    let on_screen = state_json(&m.state)["classroom"].clone();
    assert_eq!(on_screen["active"]["title"], "Classroom");
    assert_eq!(on_screen["active"]["turns"], 0);
    assert_eq!(on_screen["active"]["participants"], 4);
    assert!(on_screen["confirm_delete"].is_null());
    assert_eq!(on_screen["saved"][0]["title"], "Classroom");

    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::Leave));
    let kept = state_json(&m.state)["classroom"].clone();
    assert!(
        kept["active"].is_null(),
        "no session is on screen after leaving"
    );
    assert_eq!(kept["saved"].as_array().unwrap().len(), 1);
    assert_eq!(kept["saved"][0]["tasks"], 0);
}

#[test]
fn the_snapshot_reports_a_pending_delete_and_the_last_notice() {
    let (_vault, mut m, mut w) = started();
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::Leave));
    let id = m.state.classroom.saved[0].id;
    let msg = ClassroomMsg::Saved(studyfrog_ui::classroom_types::SavedMsg::AskDelete(id));
    send(&mut m, &mut w, Msg::Classroom(msg));
    let v = state_json(&m.state)["classroom"].clone();
    assert!(
        v["confirm_delete"].is_string(),
        "the delete waits for confirmation"
    );

    let confirm = ClassroomMsg::Saved(studyfrog_ui::classroom_types::SavedMsg::ConfirmDelete);
    send(&mut m, &mut w, Msg::Classroom(confirm));
    let v = state_json(&m.state)["classroom"].clone();
    assert_eq!(v["notice"], "Session deleted.");
    assert!(v["saved"].as_array().unwrap().is_empty());
}
