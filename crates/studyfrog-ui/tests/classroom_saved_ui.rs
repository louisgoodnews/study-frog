#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The saved classroom sessions (TASK-0295): the browser under the form, resume where it left off,
//! Markdown export and confirmed delete. The worker runs against the sample vault; no model is called
//! beyond the fake local one.
use classroom_common::{asked, classroom, online, sized, started};
use common::flow::send;
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_ui::classroom_types::{ClassroomEffect, ClassroomMsg, SavedEffect, SavedMsg};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::Model;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

mod classroom_common;
mod common;

fn saved_msg(m: &mut Model, w: &mut studyfrog_ui::exec::Worker, msg: SavedMsg) {
    send(m, w, Msg::Classroom(ClassroomMsg::Saved(msg)));
}

fn leave(m: &mut Model, w: &mut studyfrog_ui::exec::Worker) {
    send(m, w, Msg::Classroom(ClassroomMsg::Leave));
}

fn advance(m: &mut Model, w: &mut studyfrog_ui::exec::Worker) {
    send(m, w, Msg::Classroom(ClassroomMsg::Advance));
}

fn first_id(m: &Model) -> Uid {
    m.state
        .classroom
        .saved
        .first()
        .expect("one saved session")
        .id
}

/// A started session with one teacher turn, then back to the form: the session is only saved.
fn saved_one() -> (
    common::TempDir,
    Model,
    studyfrog_ui::exec::Worker,
    common::TempDir,
) {
    let (vault, mut m, mut w) = started();
    let cfg = online(&mut w);
    advance(&mut m, &mut w);
    leave(&mut m, &mut w);
    (vault, m, w, cfg)
}

#[test]
fn resuming_a_saved_session_restores_its_chat_so_the_lecture_goes_on() {
    let (_vault, mut m, mut w, _cfg) = saved_one();
    assert!(m.state.classroom.session.is_none());
    let id = first_id(&m);

    saved_msg(&mut m, &mut w, SavedMsg::Resume(id));

    let s = m
        .state
        .classroom
        .session
        .as_ref()
        .expect("the session is back on screen");
    assert_eq!(s.id, id);
    assert_eq!(s.turns.len(), 1, "the stored teacher turn is back");
    assert_eq!(s.turns[0].text, "Plants turn light into sugar.");
    assert!(!m.state.classroom.busy);
    assert!(m.state.classroom.error.is_none());

    advance(&mut m, &mut w);
    assert_eq!(
        m.state.classroom.session.as_ref().unwrap().turns.len(),
        2,
        "the lecture continues from the stored chat"
    );
}

#[test]
fn export_writes_the_transcript_as_markdown_into_the_vault() {
    let (_vault, mut m, mut w, _cfg) = saved_one();
    let id = first_id(&m);

    saved_msg(&mut m, &mut w, SavedMsg::Export(id));

    let notice = m
        .state
        .classroom
        .saved_notice
        .clone()
        .expect("the export reports");
    let path = notice
        .strip_prefix("Exported to ")
        .expect("the notice names the written file");
    assert!(
        path.contains("exports"),
        "written under the vault's exports folder"
    );
    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.starts_with("# Classroom\n"), "titled by the session");
    assert!(
        text.contains("Plants turn light into sugar."),
        "the transcript is in it"
    );
    assert!(m.state.classroom.error.is_none());
}

#[test]
fn delete_waits_for_confirmation_and_removes_the_session_only_when_confirmed() {
    let (_vault, mut m, mut w, _cfg) = saved_one();
    let id = first_id(&m);

    saved_msg(&mut m, &mut w, SavedMsg::AskDelete(id));
    assert_eq!(m.state.classroom.confirm_delete, Some(id));
    assert_eq!(m.state.classroom.saved.len(), 1, "nothing is deleted yet");

    saved_msg(&mut m, &mut w, SavedMsg::CancelDelete);
    assert!(m.state.classroom.confirm_delete.is_none());
    assert_eq!(
        m.state.classroom.saved.len(),
        1,
        "a cancelled delete keeps it"
    );

    saved_msg(&mut m, &mut w, SavedMsg::AskDelete(id));
    saved_msg(&mut m, &mut w, SavedMsg::ConfirmDelete);
    assert!(
        m.state.classroom.saved.is_empty(),
        "the confirmed delete removes it"
    );
    assert!(m.state.classroom.confirm_delete.is_none());
    assert_eq!(
        m.state.classroom.saved_notice.as_deref(),
        Some("Session deleted.")
    );

    saved_msg(&mut m, &mut w, SavedMsg::Resume(id));
    assert_eq!(
        m.state.classroom.error.as_deref(),
        Some("That session is no longer saved."),
        "a deleted session cannot be resumed"
    );
}

#[test]
fn the_browser_buttons_ask_the_right_effects_and_delete_waits_for_the_confirm_button() {
    let (_vault, mut m, _w, _cfg) = saved_one();
    m.state.screen = Screen::Classroom;
    m.state.classroom.section = studyfrog_ui::classroom_types::ClassroomSection::Saved;
    let id = first_id(&m);
    let mut h = sized(m, 960.0, 640.0);

    h.get_by_label("Resume Classroom").click();
    h.run_steps(3);
    assert!(asked(&h, |e| matches!(
        e,
        Effect::Classroom(ClassroomEffect::Saved(SavedEffect::Load { .. }))
    )));

    h.state_mut().0.state.classroom.busy = false;
    h.get_by_label("Export Classroom to Markdown").click();
    h.run_steps(3);
    assert!(asked(&h, |e| matches!(
        e,
        Effect::Classroom(ClassroomEffect::Saved(SavedEffect::Export { id: x })) if *x == id
    )));

    h.get_by_label("Delete Classroom").click();
    h.run_steps(3);
    assert!(
        h.query_by_label("Keep it").is_some(),
        "the question is shown"
    );
    h.get_by_label("Confirm delete").click();
    h.run_steps(3);
    assert!(asked(&h, |e| matches!(
        e,
        Effect::Classroom(ClassroomEffect::Saved(SavedEffect::Delete { id: x })) if *x == id
    )));
}

#[test]
fn a_session_started_from_the_form_is_listed_when_it_is_left() {
    let (_vault, mut m, _w) = started();
    assert!(
        m.state
            .classroom
            .saved
            .iter()
            .any(|s| s.title == "Classroom"),
        "the start stored the session and the list read it back"
    );
    let fx = classroom(&mut m, ClassroomMsg::Leave);
    assert!(
        fx.iter().any(|e| matches!(
            e,
            Effect::Classroom(ClassroomEffect::Saved(SavedEffect::List))
        )),
        "leaving asks for the list again"
    );
}
