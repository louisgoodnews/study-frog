#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The exam of the assistant screen.

mod common;

use common::views::open_model;
use studyfrog_agent::provider::{Protocol, Provider};
use studyfrog_service::agent::Connections;
use studyfrog_ui::agent_types::{AgentMsg, AgentTab};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Agent;
    let local = Provider {
        name: "Local".into(),
        protocol: Protocol::Ollama,
        base_url: "http://localhost:11434".into(),
        model: "llama".into(),
        allowed: false,
    };
    apply(
        &mut m,
        Msg::Agent(AgentMsg::Loaded(Connections {
            providers: vec![local],
            ..Connections::default()
        })),
    );
    m
}

fn agent(m: &mut Model, msg: AgentMsg) -> Vec<Effect> {
    apply(m, Msg::Agent(msg))
}

#[test]
fn an_exam_asks_waits_for_answers_and_records_verdicts_only_on_request() {
    let mut m = model();
    agent(&mut m, AgentMsg::Tab(AgentTab::Exam));
    agent(&mut m, AgentMsg::Folder("french".into(), true));
    agent(&mut m, AgentMsg::ExamCount("x".into()));
    assert!(agent(&mut m, AgentMsg::StartExam).is_empty());
    agent(&mut m, AgentMsg::ExamCount("2".into()));
    assert_eq!(
        agent(&mut m, AgentMsg::StartExam),
        [Effect::PrepareExam {
            folders: vec!["french".into()],
            count: 2
        }]
    );
    let items = vec![studyfrog_agent::exam::ExamItem {
        uid: "A1".into(),
        prompt: "Q?".into(),
        reference: "R".into(),
    }];
    let fx = agent(&mut m, AgentMsg::ExamPrepared(Ok(items)));
    assert!(
        matches!(&fx[..], [Effect::AgentChat { .. }]),
        "the opening turn starts the exam"
    );
    agent(&mut m, AgentMsg::ExamReplied(Ok("Question 1: Q?".into())));
    agent(&mut m, AgentMsg::ExamInput("my answer".into()));
    let fx = agent(&mut m, AgentMsg::ExamSend);
    assert!(
        matches!(&fx[..], [Effect::AgentChat { messages, .. }] if messages.len() == 3 && messages[2].content == "my answer")
    );
    agent(
        &mut m,
        AgentMsg::ExamReplied(Ok(
            "Right!\n{\"results\":[{\"id\":\"A1\",\"verdict\":\"correct\"}]}".into(),
        )),
    );
    assert_eq!(m.state.agent.exam.results.as_ref().map(Vec::len), Some(1));
    assert!(
        !m.state
            .agent
            .exam
            .chat
            .last()
            .unwrap()
            .content
            .contains("results"),
        "the JSON is not shown"
    );
    assert!(
        m.state.agent.exam.recorded.is_none(),
        "nothing is recorded by itself"
    );
    let fx = agent(&mut m, AgentMsg::RecordExam);
    assert!(matches!(&fx[..], [Effect::RecordExam(r)] if r.len() == 1));
    agent(&mut m, AgentMsg::ExamRecorded(Ok(1)));
    assert!(agent(&mut m, AgentMsg::RecordExam).is_empty(), "only once");
}
