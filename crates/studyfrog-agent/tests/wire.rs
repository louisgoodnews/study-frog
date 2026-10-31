#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Requests and answers of the three protocols, and what counts as local.

use serde_json::Value;
use studyfrog_agent::provider::{
    HttpResponse, Message, Protocol, Provider, Role, host_of, is_local, may_send,
};
use studyfrog_agent::wire::{build_request, parse_reply};

fn provider(protocol: Protocol, url: &str) -> Provider {
    Provider {
        name: "p".into(),
        protocol,
        base_url: url.into(),
        model: "m1".into(),
        allowed: false,
    }
}

fn chat() -> Vec<Message> {
    vec![
        Message {
            role: Role::User,
            content: "hi".into(),
        },
        Message {
            role: Role::Assistant,
            content: "yo".into(),
        },
    ]
}

fn body(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn ok(text: &str) -> HttpResponse {
    HttpResponse {
        status: 200,
        body: text.into(),
    }
}

#[test]
fn ollama_posts_to_api_chat_with_the_system_turn_first_and_no_streaming() {
    let r = build_request(
        &provider(Protocol::Ollama, "http://localhost:11434/"),
        None,
        "be nice",
        &chat(),
    )
    .unwrap();
    assert_eq!(r.url, "http://localhost:11434/api/chat");
    let b = body(&r.body);
    assert_eq!(
        (b["model"].as_str(), b["stream"].as_bool()),
        (Some("m1"), Some(false))
    );
    assert_eq!(b["messages"][0]["role"], "system");
    assert_eq!(b["messages"][2]["role"], "assistant");
    let reply = parse_reply(
        &provider(Protocol::Ollama, "x"),
        &ok(r#"{"message":{"role":"assistant","content":"hello"}}"#),
    )
    .unwrap();
    assert_eq!(reply, "hello");
}

#[test]
fn openai_sends_a_bearer_key_and_a_local_server_may_go_without() {
    let p = provider(Protocol::OpenAiCompatible, "https://api.openai.com");
    assert!(
        build_request(&p, None, "s", &chat())
            .unwrap_err()
            .contains("needs an API key")
    );
    let r = build_request(&p, Some("sk-1"), "s", &chat()).unwrap();
    assert_eq!(r.url, "https://api.openai.com/v1/chat/completions");
    assert!(
        r.headers
            .contains(&("Authorization".into(), "Bearer sk-1".into()))
    );
    assert!(
        build_request(
            &provider(Protocol::OpenAiCompatible, "http://127.0.0.1:8080"),
            None,
            "s",
            &chat()
        )
        .is_ok()
    );
    let reply = parse_reply(&p, &ok(r#"{"choices":[{"message":{"content":"ok"}}]}"#)).unwrap();
    assert_eq!(reply, "ok");
}

#[test]
fn anthropic_puts_the_system_text_aside_and_joins_the_text_blocks() {
    let p = provider(Protocol::Anthropic, "https://api.anthropic.com");
    assert!(build_request(&p, Some(""), "s", &chat()).is_err());
    let r = build_request(&p, Some("k"), "rules", &chat()).unwrap();
    assert_eq!(r.url, "https://api.anthropic.com/v1/messages");
    assert!(
        r.headers.contains(&("x-api-key".into(), "k".into()))
            && r.headers.iter().any(|h| h.0 == "anthropic-version")
    );
    let b = body(&r.body);
    assert_eq!(
        (
            b["system"].as_str(),
            b["max_tokens"].as_u64(),
            b["messages"].as_array().map(Vec::len)
        ),
        (Some("rules"), Some(4096), Some(2))
    );
    let reply = parse_reply(&p, &ok(r#"{"content":[{"type":"text","text":"a"},{"type":"tool_use"},{"type":"text","text":"b"}]}"#)).unwrap();
    assert_eq!(reply, "ab");
}

#[test]
fn failures_are_sentences_that_never_contain_the_key() {
    let p = provider(Protocol::Anthropic, "https://api.anthropic.com");
    let e = parse_reply(
        &p,
        &HttpResponse {
            status: 401,
            body: r#"{"error":{"message":"invalid x-api-key"}}"#.into(),
        },
    )
    .unwrap_err();
    assert!(
        e.contains("401") && e.contains("key was refused") && e.contains("invalid x-api-key"),
        "{e}"
    );
    assert!(
        parse_reply(
            &p,
            &HttpResponse {
                status: 404,
                body: "no".into()
            }
        )
        .unwrap_err()
        .contains("model name")
    );
    assert!(
        parse_reply(&p, &ok("<html>"))
            .unwrap_err()
            .contains("not JSON")
    );
    assert!(
        parse_reply(&p, &ok(r#"{"content":[]}"#))
            .unwrap_err()
            .contains("without any text")
    );
    let long = parse_reply(
        &p,
        &HttpResponse {
            status: 500,
            body: "x".repeat(5000),
        },
    )
    .unwrap_err();
    assert!(long.len() < 500);
}

#[test]
fn local_servers_are_recognised_and_remote_ones_need_permission() {
    for (url, host) in [
        ("https://api.openai.com/v1", "api.openai.com"),
        ("http://localhost:11434", "localhost"),
        ("http://[::1]:8080/x", "::1"),
        ("http://u:p@host:1/", "host"),
    ] {
        assert_eq!(host_of(url), host);
    }
    let mut p = provider(Protocol::Ollama, "http://localhost:11434");
    assert!(is_local(&p) && may_send(&p));
    p.base_url = "https://ollama.example.com".into();
    assert!(!is_local(&p) && !may_send(&p));
    p.allowed = true;
    assert!(may_send(&p));
    assert!(
        !is_local(&provider(Protocol::Ollama, "http://localhost.evil.com")),
        "only the exact host counts"
    );
}
