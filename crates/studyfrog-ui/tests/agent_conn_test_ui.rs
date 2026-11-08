#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! "Test connection" on the Connections tab (STORY-0083, STORY-0084): the button, the worker's
//! request through the injected transport, and the outcome shown under the connection.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_agent::provider::{HttpRequest, HttpResponse, Protocol, Provider};
use studyfrog_service::agent::{Connections, save_providers, set_key};
use studyfrog_ui::agent_conn_test::TestMsg;
use studyfrog_ui::agent_mcp_types::AgentEffect;
use studyfrog_ui::agent_types::AgentMsg;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

const KEY: &str = "sk-ui-secret-42";

fn provider(name: &str, protocol: Protocol, allowed: bool) -> Provider {
    Provider {
        name: name.into(),
        protocol,
        base_url: studyfrog_agent::provider::default_base_url(protocol).into(),
        model: studyfrog_agent::provider::default_models(protocol)[0].into(),
        allowed,
    }
}

fn model(providers: Vec<Provider>) -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Agent;
    let c = Connections {
        providers,
        ..Connections::default()
    };
    apply(&mut m, Msg::Agent(AgentMsg::Loaded(c)));
    m
}

fn gemini_ok(_: &HttpRequest) -> Result<HttpResponse, String> {
    Ok(HttpResponse {
        status: 200,
        body: r#"{"candidates":[{"content":{"parts":[{"text":"OK"}]}}]}"#.into(),
        headers: Vec::new(),
    })
}

fn refused(_: &HttpRequest) -> Result<HttpResponse, String> {
    Ok(HttpResponse {
        status: 401,
        body: format!("bad key {KEY}"),
        headers: Vec::new(),
    })
}

#[test]
fn the_button_asks_the_worker_and_the_outcome_appears_under_the_connection() {
    let m = model(vec![provider("Gem", Protocol::Gemini, true)]);
    let mut h = harness(m);
    h.run_steps(4);
    h.get_by_label("Test connection to Gem").click();
    h.run_steps(2);
    let asked = h
        .state()
        .1
        .iter()
        .any(|e| matches!(e, Effect::Agent(AgentEffect::TestConnection(p)) if p.name == "Gem"));
    assert!(asked, "the test goes to the worker: {:?}", h.state().1);
    assert!(h.query_by_label("Testing...").is_some());
    let done = TestMsg::Done("Gem".into(), Ok("Gem answered: it works.".into()));
    apply(&mut h.state_mut().0, Msg::Agent(AgentMsg::Test(done)));
    h.run_steps(3);
    assert!(h.query_by_label_contains("Gem answered").is_some());
    let failed = TestMsg::Done("Gem".into(), Err("Gem answered 429 (wait)".into()));
    apply(&mut h.state_mut().0, Msg::Agent(AgentMsg::Test(failed)));
    h.run_steps(3);
    assert!(
        h.query_by_label_contains("Connection test failed")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("it works").is_none(),
        "only the last outcome"
    );
    assert_all_named(&h, "connections with a test outcome");
}

#[test]
fn a_remote_connection_not_yet_allowed_cannot_be_tested() {
    let mut h = harness(model(vec![provider("Deep", Protocol::DeepSeek, false)]));
    h.run_steps(4);
    h.get_by_label("Test connection to Deep").click();
    h.run_steps(2);
    let asked = h
        .state()
        .1
        .iter()
        .any(|e| matches!(e, Effect::Agent(AgentEffect::TestConnection(_))));
    assert!(!asked, "a disabled button does nothing");
    assert!(h.state().0.state.agent.conn_tests.running.is_empty());
}

#[test]
fn the_worker_tests_through_the_transport_and_never_shows_the_key() {
    let dir = common::temp_dir("conn-test-worker");
    let gem = provider("Gem", Protocol::Gemini, true);
    save_providers(&dir.path, std::slice::from_ref(&gem)).unwrap();
    set_key(&dir.path, "Gem", Some(KEY)).unwrap();
    let mut w = worker_new(Some(dir.path.clone()));
    let mut outcome = |transport: studyfrog_agent::provider::Transport| {
        w.transport = Some(transport);
        let effect = Effect::Agent(AgentEffect::TestConnection(gem.clone()));
        match execute(&mut w, effect, common::NOW_NS).pop() {
            Some(Msg::Agent(AgentMsg::Test(TestMsg::Done(name, r)))) if name == "Gem" => r,
            other => panic!("unexpected {other:?}"),
        }
    };
    assert!(outcome(gemini_ok).unwrap().contains("works"));
    let e = outcome(refused).unwrap_err();
    assert!(e.contains("401") && !e.contains(KEY), "{e}");
    let mut none = worker_new(Some(dir.path.clone()));
    let effect = Effect::Agent(AgentEffect::TestConnection(gem));
    let msg = format!("{:?}", execute(&mut none, effect, common::NOW_NS));
    assert!(msg.contains("cannot reach the network"), "{msg}");
}

#[test]
fn a_rate_limit_during_an_exam_or_a_prompt_ends_the_wait_and_shows_the_sentence() {
    let mut m = model(vec![provider("Deep", Protocol::DeepSeek, true)]);
    m.state.agent.busy = true;
    let why = "Deep answered 429 (too many requests; wait a moment): Rate limit reached";
    apply(&mut m, Msg::Agent(AgentMsg::ExamReplied(Err(why.into()))));
    assert!(!m.state.agent.busy);
    assert_eq!(m.state.agent.error.as_deref(), Some(why));
    m.state.prompt_run.busy = true;
    let replied = studyfrog_ui::prompt_run::PromptRunMsg::Replied(Err(why.into()));
    apply(
        &mut m,
        Msg::Prompts(studyfrog_ui::prompts_ui::PromptsMsg::Run(replied)),
    );
    assert!(!m.state.prompt_run.busy);
    assert_eq!(m.state.prompt_run.error.as_deref(), Some(why));
}
