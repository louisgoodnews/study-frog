#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)] // shared test helpers
//! Helpers for the Classroom screen tests: a started session, a chat harness and a fake local model.

use crate::common::flow::{opened, send};
use crate::common::views::App;
use crate::common::{TempDir, temp_dir};
use eframe::egui;
use egui_kittest::Harness;
use studyfrog_agent::provider::{HttpRequest, HttpResponse, Protocol, Provider};
use studyfrog_core::classroom::{SourceKind, SourceRef};
use studyfrog_core::classroom_ops::append_turn;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_ui::classroom_types::ClassroomMsg;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::Worker;
use studyfrog_ui::model::{Model, apply, frame};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

pub fn provider() -> Provider {
    Provider {
        name: "LocalLLM".into(),
        protocol: Protocol::Ollama,
        base_url: "http://127.0.0.1:11434".into(),
        model: "llama3".into(),
        allowed: true,
    }
}

/// The sample vault's French stack, which has notes to teach from.
pub fn french(m: &Model) -> Uid {
    m.state.vault.as_ref().unwrap().overview.stacks[0].uid
}

pub fn connect(m: &mut Model) {
    m.state.agent.connections.providers = vec![provider()];
}

pub fn classroom(m: &mut Model, msg: ClassroomMsg) -> Vec<Effect> {
    apply(m, Msg::Classroom(msg))
}

/// A fake local model that answers every turn with the same sentence.
pub fn teacher_reply(_: &HttpRequest) -> Result<HttpResponse, String> {
    Ok(HttpResponse {
        status: 200,
        body: r#"{"message":{"role":"assistant","content":"Plants turn light into sugar."}}"#
            .into(),
        headers: Vec::new(),
    })
}

/// A started session with a French source, a teacher, the user and two simulated students.
pub fn started() -> (TempDir, Model, Worker) {
    let (vault, mut m, mut w) = opened("classroom-started");
    connect(&mut m);
    let uid = french(&m);
    classroom(
        &mut m,
        ClassroomMsg::ToggleSource(SourceRef {
            kind: SourceKind::Stack,
            uid,
        }),
    );
    classroom(&mut m, ClassroomMsg::Students(2));
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::Start));
    assert!(
        m.state.classroom.session.is_some(),
        "the session is started"
    );
    (vault, m, w)
}

/// Builds a harness on the chat with one turn from each kind of speaker.
pub fn chat_harness(mut m: Model) -> Harness<'static, App> {
    let session = m.state.classroom.session.as_mut().unwrap();
    append_turn(session, 0, UnixMs(1), "Welcome to the lecture.").unwrap();
    append_turn(session, 1, UnixMs(2), "What is a noun?").unwrap();
    append_turn(session, 2, UnixMs(3), "Why does it matter?").unwrap();
    m.state.screen = Screen::Classroom;
    let mut h = crate::common::views::harness(m);
    h.run_steps(4);
    h
}

pub fn sized(m: Model, w: f32, h: f32) -> Harness<'static, App> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(w, h))
        .build_ui_state(
            |ui, (model, effects): &mut App| effects.extend(frame(model, ui)),
            (m, Vec::new()),
        );
    harness.run_steps(4);
    harness
}

pub fn asked(h: &Harness<'static, App>, want: impl Fn(&Effect) -> bool) -> bool {
    h.state().1.iter().any(want)
}

/// A worker that can reach the fake local model, with a config folder for it.
pub fn online(w: &mut Worker) -> TempDir {
    let cfg = temp_dir("classroom-work-cfg");
    w.config_dir = Some(cfg.path.clone());
    w.transport = Some(teacher_reply);
    cfg
}

/// The id of the assignment added last.
pub fn last_task(m: &Model) -> Uid {
    m.state
        .classroom
        .session
        .as_ref()
        .unwrap()
        .assignments
        .last()
        .unwrap()
        .id
}

/// The teacher sets a solo task for the user (participant 1) through the worker.
pub fn solo_task(m: &mut Model, w: &mut Worker) -> Uid {
    send(m, w, Msg::Classroom(ClassroomMsg::AssignTask));
    last_task(m)
}

/// The teacher sets group work through the worker; the user is in one group.
pub fn group_task(m: &mut Model, w: &mut Worker) -> Uid {
    send(m, w, Msg::Classroom(ClassroomMsg::AssignGroups));
    last_task(m)
}

pub fn open(m: &mut Model, w: &mut Worker, id: Uid) {
    send(m, w, Msg::Classroom(ClassroomMsg::OpenTask(Some(id))));
}

pub fn stored_turns(m: &Model) -> Vec<(usize, String)> {
    m.state
        .classroom
        .session
        .as_ref()
        .unwrap()
        .turns
        .iter()
        .map(|t| (t.speaker, t.text.clone()))
        .collect()
}
