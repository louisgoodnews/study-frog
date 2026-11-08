#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! "Test connection" (STORY-0083, STORY-0084): one tiny request through the given transport, the
//! same consent rule as every question, and an outcome that never shows the API key.

use std::path::PathBuf;
use std::sync::Mutex;

use studyfrog_agent::provider::{HttpRequest, HttpResponse, Protocol, Provider};
use studyfrog_service::agent::{Ask, save_providers, set_key, test_connection};

static SEEN: Mutex<Vec<HttpRequest>> = Mutex::new(Vec::new());
const KEY: &str = "sk-very-secret-123";

fn reply(status: u16, body: &str) -> Result<HttpResponse, String> {
    Ok(HttpResponse {
        status,
        body: body.into(),
        headers: Vec::new(),
    })
}

fn gemini_ok(req: &HttpRequest) -> Result<HttpResponse, String> {
    SEEN.lock().unwrap().push(req.clone());
    reply(
        200,
        r#"{"candidates":[{"content":{"parts":[{"text":"OK"}]}}]}"#,
    )
}

fn deepseek_ok(req: &HttpRequest) -> Result<HttpResponse, String> {
    SEEN.lock().unwrap().push(req.clone());
    reply(200, r#"{"choices":[{"message":{"content":"OK"}}]}"#)
}

fn echoes_the_key(_: &HttpRequest) -> Result<HttpResponse, String> {
    reply(401, &format!(r#"{{"error":"invalid key {KEY}"}}"#))
}

fn rate_limited(_: &HttpRequest) -> Result<HttpResponse, String> {
    reply(429, r#"{"error":{"message":"Rate limit reached"}}"#)
}

fn unreachable(_: &HttpRequest) -> Result<HttpResponse, String> {
    Err("could not connect to api.deepseek.com".into())
}

fn never(_: &HttpRequest) -> Result<HttpResponse, String> {
    panic!("nothing may be sent before the user allows it")
}

fn scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sf-conn-test-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn provider(protocol: Protocol, allowed: bool) -> Provider {
    Provider {
        name: "Remote".into(),
        protocol,
        base_url: studyfrog_agent::provider::default_base_url(protocol).into(),
        model: studyfrog_agent::provider::default_models(protocol)[0].into(),
        allowed,
    }
}

fn setup(label: &str, p: &Provider) -> PathBuf {
    let dir = scratch(label);
    save_providers(&dir, std::slice::from_ref(p)).unwrap();
    set_key(&dir, &p.name, Some(KEY)).unwrap();
    dir
}

fn ctx(dir: &std::path::Path) -> Ask<'_> {
    Ask {
        config_dir: dir,
        log_dir: Some(dir),
        now_ms: 1,
    }
}

#[test]
fn gemini_and_deepseek_answer_through_their_own_wire_with_the_key_in_a_header() {
    for (protocol, transport, path, header) in [
        (
            Protocol::Gemini,
            gemini_ok as studyfrog_agent::provider::Transport,
            "/v1beta/models/gemini-1.5-flash:generateContent",
            "x-goog-api-key",
        ),
        (
            Protocol::DeepSeek,
            deepseek_ok,
            "/chat/completions",
            "Authorization",
        ),
    ] {
        let p = provider(protocol, true);
        let dir = setup(&format!("{protocol:?}"), &p);
        let said = test_connection(transport, &p, ctx(&dir)).unwrap();
        assert!(said.contains("works"), "{said}");
        assert!(!said.contains(KEY));
        let req = SEEN.lock().unwrap().pop().unwrap();
        assert!(req.url.ends_with(path), "{}", req.url);
        let sent = req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(header));
        assert!(sent.is_some_and(|(_, v)| v.contains(KEY)));
        assert!(!req.url.contains(KEY) && !req.body.contains(KEY));
        assert!(req.body.len() < 400, "a tiny request: {}", req.body);
        let log = std::fs::read_to_string(dir.join("agent.log")).unwrap();
        assert!(!log.contains(KEY), "the log never holds the key: {log}");
    }
}

#[test]
fn a_remote_connection_is_not_tested_before_the_user_allows_it() {
    let p = provider(Protocol::DeepSeek, false);
    let dir = setup("consent", &p);
    let e = test_connection(never, &p, ctx(&dir)).unwrap_err();
    assert!(e.contains("allow"), "{e}");
}

#[test]
fn errors_become_sentences_without_the_key_even_when_the_server_echoes_it() {
    let p = provider(Protocol::Gemini, true);
    let dir = setup("errors", &p);
    let refused = test_connection(echoes_the_key, &p, ctx(&dir)).unwrap_err();
    assert!(
        refused.contains("401") && refused.contains("refused"),
        "{refused}"
    );
    assert!(!refused.contains(KEY), "{refused}");
    let busy = test_connection(rate_limited, &p, ctx(&dir)).unwrap_err();
    assert!(busy.contains("429") && busy.contains("wait"), "{busy}");
    let down = test_connection(unreachable, &p, ctx(&dir)).unwrap_err();
    assert!(down.contains("could not connect"), "{down}");
}
