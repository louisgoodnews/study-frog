#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The requests and answers of Google Gemini and DeepSeek.

use serde_json::Value;
use studyfrog_agent::provider::{HttpResponse, Message, Protocol, Provider, Role, is_local};
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
fn gemini_posts_to_generate_content_with_the_key_in_a_header_and_never_in_the_address() {
    let p = provider(
        Protocol::Gemini,
        "https://generativelanguage.googleapis.com/",
    );
    let r = build_request(&p, Some("SECRET-KEY"), "be nice", &chat()).unwrap();
    assert_eq!(
        r.url,
        "https://generativelanguage.googleapis.com/v1beta/models/m1:generateContent"
    );
    assert!(
        !r.url.contains("SECRET-KEY"),
        "the key must not reach a URL log"
    );
    assert!(
        r.headers
            .iter()
            .any(|(k, v)| k == "x-goog-api-key" && v == "SECRET-KEY")
    );
    let b = body(&r.body);
    assert_eq!(b["system_instruction"]["parts"][0]["text"], "be nice");
    assert_eq!(b["contents"][0]["role"], "user");
    assert_eq!(b["contents"][1]["role"], "model");
    assert_eq!(b["contents"][1]["parts"][0]["text"], "yo");
    assert!(b.get("model").is_none(), "the model is in the address");
}

#[test]
fn gemini_without_a_key_is_refused_before_anything_is_sent() {
    let p = provider(
        Protocol::Gemini,
        "https://generativelanguage.googleapis.com",
    );
    let err = build_request(&p, None, "s", &chat()).unwrap_err();
    assert!(err.contains("API key"), "{err}");
}

#[test]
fn gemini_answers_are_joined_and_safety_stops_say_why_without_the_text() {
    let p = provider(
        Protocol::Gemini,
        "https://generativelanguage.googleapis.com",
    );
    let reply = r#"{"candidates":[{"content":{"parts":[{"text":"Hel"},{"text":"lo"}]},"finishReason":"STOP"}]}"#;
    assert_eq!(parse_reply(&p, &ok(reply)).unwrap(), "Hello");
    let safety = r#"{"candidates":[{"finishReason":"SAFETY"}]}"#;
    let err = parse_reply(&p, &ok(safety)).unwrap_err();
    assert!(err.contains("SAFETY"), "{err}");
    let blocked = r#"{"promptFeedback":{"blockReason":"OTHER"},"candidates":[]}"#;
    assert!(parse_reply(&p, &ok(blocked)).unwrap_err().contains("OTHER"));
    let empty = r#"{"candidates":[{"content":{"parts":[]},"finishReason":"STOP"}]}"#;
    assert!(
        parse_reply(&p, &ok(empty))
            .unwrap_err()
            .contains("without any text")
    );
}

#[test]
fn gemini_errors_show_the_providers_message_and_hint_at_rate_limits() {
    let p = provider(
        Protocol::Gemini,
        "https://generativelanguage.googleapis.com",
    );
    let limited = HttpResponse {
        status: 429,
        body: r#"{"error":{"message":"Quota exceeded"}}"#.into(),
    };
    let err = parse_reply(&p, &limited).unwrap_err();
    assert!(
        err.contains("429") && err.contains("Quota exceeded") && err.contains("wait"),
        "{err}"
    );
}

#[test]
fn deepseek_posts_chat_completions_with_a_bearer_key_and_ignores_the_reasoning() {
    let p = provider(Protocol::DeepSeek, "https://api.deepseek.com/");
    let r = build_request(&p, Some("k1"), "be nice", &chat()).unwrap();
    assert_eq!(r.url, "https://api.deepseek.com/chat/completions");
    assert!(
        r.headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer k1")
    );
    let b = body(&r.body);
    assert_eq!(b["messages"][0]["role"], "system");
    assert_eq!(b["messages"][2]["role"], "assistant");
    assert!(
        build_request(&p, None, "s", &chat()).is_err(),
        "a key is required"
    );
    let reply =
        r#"{"choices":[{"message":{"content":"The answer.","reasoning_content":"let me think"}}]}"#;
    assert_eq!(parse_reply(&p, &ok(reply)).unwrap(), "The answer.");
}

#[test]
fn the_new_protocols_have_names_addresses_and_suggested_models() {
    use studyfrog_agent::provider::{default_base_url, default_models, protocol_from_name};
    assert_eq!(protocol_from_name("gemini"), Some(Protocol::Gemini));
    assert_eq!(protocol_from_name(" deepseek "), Some(Protocol::DeepSeek));
    assert_eq!(
        default_base_url(Protocol::DeepSeek),
        "https://api.deepseek.com"
    );
    assert_eq!(
        default_models(Protocol::DeepSeek),
        ["deepseek-chat", "deepseek-reasoner"]
    );
    assert_eq!(default_models(Protocol::Gemini)[0], "gemini-1.5-flash");
    assert!(default_models(Protocol::Ollama).is_empty());
    assert!(!is_local(&provider(
        Protocol::DeepSeek,
        "https://api.deepseek.com"
    )));
}
