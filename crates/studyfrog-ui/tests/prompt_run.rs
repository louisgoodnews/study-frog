#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0207: a standard prompt through the assistant: shown before it is sent, sent only after Send, the answer
//! imported through the preview, the problems sent back the same way.

mod common;

use common::flow::opened;
use studyfrog_agent::provider::{Protocol, Provider, Role};
use studyfrog_service::agent::Connections;
use studyfrog_ui::agent_types::{AgentMsg, ChatPurpose};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::prompt_run::PromptRunMsg;
use studyfrog_ui::prompts_ui::PromptsMsg;
use studyfrog_ui::state::Screen;

fn run(m: &mut Model, msg: PromptRunMsg) -> Vec<Effect> {
    apply(m, Msg::Prompts(PromptsMsg::Run(msg)))
}

fn connect(m: &mut Model, base: &str, allowed: bool) {
    let p = Provider {
        name: "Chat".into(),
        protocol: Protocol::Ollama,
        base_url: base.into(),
        model: "m".into(),
        allowed,
    };
    apply(
        m,
        Msg::Agent(AgentMsg::Loaded(Connections {
            providers: vec![p],
            ..Connections::default()
        })),
    );
    apply(m, Msg::Agent(AgentMsg::Select("Chat".into())));
}

fn file(label: &str, text: &str) -> String {
    let d = std::env::temp_dir().join(format!("sf-prompt-run-{label}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let p = d.join("notes.txt");
    std::fs::write(&p, text).unwrap();
    p.display().to_string()
}

#[test]
fn the_prompt_and_the_file_are_shown_and_only_a_press_of_send_sends_them() {
    let (_v, mut m, mut w) = opened("prompt-run");
    m.state.screen = Screen::Agent;
    connect(&mut m, "http://localhost:11434", false);
    run(&mut m, PromptRunMsg::Choose("flashcards"));
    run(
        &mut m,
        PromptRunMsg::Path(file("a", "The mitochondrion makes ATP.")),
    );
    let fx = run(&mut m, PromptRunMsg::Prepare);
    common::run_to_idle(&mut m, &mut w, fx);
    let out = m.state.prompt_run.outgoing.clone().expect("shown first");
    assert_eq!(out.len(), 1);
    assert!(out[0].content.contains("Create flashcards") || out[0].content.contains("flashcards"));
    assert!(out[0].content.contains("`notes.txt`") && out[0].content.contains("mitochondrion"));
    assert!(m.state.prompt_run.convo.is_empty(), "nothing went out yet");

    let fx = run(&mut m, PromptRunMsg::Send);
    let [
        Effect::AgentChat {
            messages, purpose, ..
        },
    ] = fx.as_slice()
    else {
        panic!("{fx:?}")
    };
    assert_eq!((messages, purpose), (&out, &ChatPurpose::Prompt));
    assert!(m.state.prompt_run.outgoing.is_none() && m.state.prompt_run.busy);
}

#[test]
fn a_remote_connection_that_was_not_allowed_gets_nothing() {
    let (_v, mut m, mut w) = opened("prompt-run-remote");
    connect(&mut m, "https://api.example.com/v1", false);
    run(&mut m, PromptRunMsg::Choose("notes"));
    run(&mut m, PromptRunMsg::Path(file("b", "text")));
    let fx = run(&mut m, PromptRunMsg::Prepare);
    common::run_to_idle(&mut m, &mut w, fx);
    assert!(run(&mut m, PromptRunMsg::Send).is_empty());
    assert!(
        m.state
            .prompt_run
            .error
            .as_deref()
            .unwrap()
            .contains("Allow sending")
    );
    assert!(
        m.state.prompt_run.outgoing.is_some(),
        "still waiting, nothing sent"
    );
}

#[test]
fn the_file_is_checked_before_anything_is_shown() {
    let (_v, mut m, mut w) = opened("prompt-run-bad");
    connect(&mut m, "http://localhost:11434", false);
    let fx = run(&mut m, PromptRunMsg::Prepare);
    assert!(
        fx.is_empty() && m.state.prompt_run.error.is_some(),
        "needs a prompt and a file"
    );
    run(&mut m, PromptRunMsg::Choose("notes"));
    run(&mut m, PromptRunMsg::Path("/nonexistent/x.txt".into()));
    let fx = run(&mut m, PromptRunMsg::Prepare);
    common::run_to_idle(&mut m, &mut w, fx);
    assert!(m.state.prompt_run.outgoing.is_none() && !m.state.prompt_run.busy);
    assert!(
        m.state
            .prompt_run
            .error
            .as_deref()
            .unwrap()
            .contains("x.txt")
    );
}

#[test]
fn the_answer_is_imported_through_the_preview_and_its_problems_go_back_after_send() {
    let (_v, mut m, mut w) = opened("prompt-run-fix");
    connect(&mut m, "http://localhost:11434", false);
    run(&mut m, PromptRunMsg::Choose("flashcards"));
    run(&mut m, PromptRunMsg::Path(file("c", "cats and dogs")));
    let fx = run(&mut m, PromptRunMsg::Prepare);
    common::run_to_idle(&mut m, &mut w, fx);
    run(&mut m, PromptRunMsg::Send);
    let reply = "Sure!\n```csv\nstack,kind,title,front,back,tags\nanimals,card,,What is a cat?,A feline,\nanimals,card,,,empty front,\n```\n";
    run(&mut m, PromptRunMsg::Replied(Ok(reply.into())));
    assert_eq!(m.state.prompt_run.reply.as_deref(), Some(reply));

    let fx = run(&mut m, PromptRunMsg::ImportReply);
    common::run_to_idle(&mut m, &mut w, fx);
    let csv = m
        .state
        .transfer
        .preview
        .clone()
        .expect("the preview is open")
        .csv
        .expect("read as CSV");
    assert_eq!(csv.problems.len(), 1);

    let fx = run(&mut m, PromptRunMsg::AskToFix);
    assert!(
        fx.iter().all(|e| !matches!(e, Effect::AgentChat { .. })),
        "shown first, not sent"
    );
    let out = m.state.prompt_run.outgoing.clone().unwrap();
    assert_eq!(out.len(), 3, "the request, the answer and the problems");
    assert_eq!(out[1].role, Role::Assistant);
    assert!(
        out[2].content.contains("row 3, column front"),
        "{}",
        out[2].content
    );
    assert_eq!(m.state.screen, Screen::Agent);
    let fx = run(&mut m, PromptRunMsg::Send);
    assert!(matches!(fx.as_slice(), [Effect::AgentChat { .. }]));
}

#[test]
fn an_answer_without_csv_or_with_a_problem_is_not_imported() {
    let (_v, mut m, _w) = opened("prompt-run-none");
    run(
        &mut m,
        PromptRunMsg::Replied(Ok("PROBLEM: the file is empty".into())),
    );
    assert!(run(&mut m, PromptRunMsg::ImportReply).is_empty());
    assert!(
        m.state
            .prompt_run
            .error
            .as_deref()
            .unwrap()
            .contains("the file is empty")
    );
    run(&mut m, PromptRunMsg::Replied(Ok("just words".into())));
    assert!(run(&mut m, PromptRunMsg::ImportReply).is_empty());
    assert!(
        m.state
            .prompt_run
            .error
            .as_deref()
            .unwrap()
            .contains("no CSV block")
    );
}
