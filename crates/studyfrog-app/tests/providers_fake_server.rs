#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Gemini and DeepSeek (STORY-0083, STORY-0084) through the application's real HTTP sender
//! (`studyfrog_net::send`) and the worker, against a fake server on 127.0.0.1 that answers in each
//! provider's documented wire format. No real endpoint is contacted.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread::JoinHandle;

use studyfrog_agent::provider::{Message, Protocol, Provider, Role, default_models};
use studyfrog_ui::agent_conn_test::TestMsg;
use studyfrog_ui::agent_mcp_types::AgentEffect;
use studyfrog_ui::agent_types::{AgentMsg, ChatPurpose};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, execute, worker_new};
use studyfrog_ui::msg::Msg;

const KEY: &str = "sk-fake-server-777";
const NOW: i64 = 1_700_000_000_000_000_000;

fn read_request(conn: &mut std::net::TcpStream) -> String {
    let (mut seen, mut buf) = (Vec::new(), [0_u8; 4096]);
    loop {
        let n = conn.read(&mut buf).unwrap();
        seen.extend_from_slice(&buf[..n]);
        let text = String::from_utf8_lossy(&seen).into_owned();
        let Some((head, rest)) = text.split_once("\r\n\r\n") else {
            continue;
        };
        let want: usize = head
            .lines()
            .find_map(|l| {
                l.to_lowercase()
                    .strip_prefix("content-length:")
                    .map(|v| v.trim().parse().unwrap())
            })
            .unwrap_or(0);
        if rest.len() >= want || n == 0 {
            return text;
        }
    }
}

/// Answer each request in turn with (status line, body); returns the address and the requests.
fn serve(replies: Vec<(&'static str, String)>) -> (String, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = format!("http://{}", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        let mut seen = Vec::new();
        for (status, body) in replies {
            let (mut conn, _) = listener.accept().unwrap();
            seen.push(read_request(&mut conn));
            let reply = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            conn.write_all(reply.as_bytes()).unwrap();
        }
        seen
    });
    (addr, handle)
}

fn worker_with(label: &str, p: &Provider) -> Worker {
    let dir = std::env::temp_dir().join(format!("sf-fake-provider-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut w = worker_new(Some(dir));
    w.transport = Some(studyfrog_net::send);
    let save = Effect::SaveConnection {
        provider: p.clone(),
        key: Some(KEY.into()),
    };
    let saved = format!("{:?}", execute(&mut w, save, NOW));
    assert!(saved.contains("Saved(Ok"), "{saved}");
    w
}

fn provider(protocol: Protocol, base_url: &str) -> Provider {
    Provider {
        name: format!("{protocol:?}"),
        protocol,
        base_url: base_url.into(),
        model: default_models(protocol)[0].into(),
        allowed: true,
    }
}

fn chat(w: &mut Worker, p: &Provider, purpose: ChatPurpose) -> Msg {
    let messages = vec![Message {
        role: Role::User,
        content: "What is spacing?".into(),
    }];
    let effect = Effect::AgentChat {
        provider: p.clone(),
        system: "Examiner".into(),
        messages,
        purpose,
    };
    execute(w, effect, NOW).pop().unwrap()
}

fn tested(w: &mut Worker, p: &Provider) -> Result<String, String> {
    match execute(
        w,
        Effect::Agent(AgentEffect::TestConnection(p.clone())),
        NOW,
    )
    .pop()
    {
        Some(Msg::Agent(AgentMsg::Test(TestMsg::Done(_, r)))) => r,
        other => panic!("unexpected {other:?}"),
    }
}

fn gemini_body(text: &str) -> String {
    format!(
        r#"{{"candidates":[{{"content":{{"role":"model","parts":[{{"text":"{text}"}}]}},"finishReason":"STOP"}}]}}"#
    )
}

fn deepseek_body(text: &str) -> String {
    format!(
        r#"{{"choices":[{{"message":{{"role":"assistant","content":"{text}","reasoning_content":"hidden"}}}}]}}"#
    )
}

#[test]
fn gemini_is_tested_and_asked_over_http_with_its_path_and_key_header() {
    let (addr, seen) = serve(vec![
        ("200 OK", gemini_body("OK")),
        ("200 OK", gemini_body("Spacing spreads reviews out.")),
        (
            "400 Bad Request",
            format!(r#"{{"error":{{"message":"API key not valid: {KEY}"}}}}"#),
        ),
    ]);
    let p = provider(Protocol::Gemini, &addr);
    let mut w = worker_with("gemini", &p);
    assert!(tested(&mut w, &p).unwrap().contains("works"));
    let answer = format!("{:?}", chat(&mut w, &p, ChatPurpose::Exam));
    assert!(
        answer.contains("ExamReplied(Ok(\"Spacing spreads reviews out.\"))"),
        "{answer}"
    );
    let e = tested(&mut w, &p).unwrap_err();
    assert!(e.contains("400") && !e.contains(KEY), "{e}");
    let wire = seen.join().unwrap();
    assert!(
        wire[0].starts_with("POST /v1beta/models/gemini-1.5-flash:generateContent "),
        "{}",
        wire[0]
    );
    assert!(
        wire.iter()
            .all(|r| r.to_lowercase().contains(&format!("x-goog-api-key: {KEY}")))
    );
    assert!(
        wire.iter()
            .all(|r| !r.split_once('\n').unwrap().0.contains(KEY)),
        "no key in the URL"
    );
}

#[test]
fn deepseek_serves_exams_and_prompt_actions_and_rate_limits_end_gracefully() {
    let (addr, seen) = serve(vec![
        ("200 OK", deepseek_body("OK")),
        ("200 OK", deepseek_body("Correct.")),
        ("200 OK", deepseek_body("front,back")),
        (
            "429 Too Many Requests",
            r#"{"error":{"message":"Rate limit reached"}}"#.into(),
        ),
        (
            "429 Too Many Requests",
            r#"{"error":{"message":"Rate limit reached"}}"#.into(),
        ),
        ("503 Service Unavailable", "upstream overloaded".into()),
    ]);
    let p = provider(Protocol::DeepSeek, &addr);
    let mut w = worker_with("deepseek", &p);
    assert!(tested(&mut w, &p).unwrap().contains("works"));
    let exam = format!("{:?}", chat(&mut w, &p, ChatPurpose::Exam));
    assert!(exam.contains("ExamReplied(Ok(\"Correct.\"))"), "{exam}");
    let prompt = format!("{:?}", chat(&mut w, &p, ChatPurpose::Prompt));
    assert!(prompt.contains("Replied(Ok(\"front,back\"))"), "{prompt}");
    let limited = format!("{:?}", chat(&mut w, &p, ChatPurpose::Exam));
    assert!(
        limited.contains("ExamReplied(Err(") && limited.contains("429"),
        "{limited}"
    );
    assert!(
        limited.contains("wait a moment") && !limited.contains(KEY),
        "{limited}"
    );
    let limited = format!("{:?}", chat(&mut w, &p, ChatPurpose::Prompt));
    assert!(
        limited.contains("Replied(Err(") && limited.contains("429"),
        "{limited}"
    );
    assert!(tested(&mut w, &p).unwrap_err().contains("503"));
    let wire = seen.join().unwrap();
    assert!(
        wire.iter()
            .all(|r| r.starts_with("POST /chat/completions ")),
        "{wire:?}"
    );
    assert!(wire.iter().all(|r| r.contains(&format!("Bearer {KEY}"))));
}

#[test]
fn a_provider_that_is_not_running_is_a_sentence_not_a_hang() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let p = provider(Protocol::DeepSeek, &addr);
    let mut w = worker_with("dead", &p);
    let e = tested(&mut w, &p).unwrap_err();
    assert!(e.contains("connect") && !e.contains(KEY), "{e}");
}
