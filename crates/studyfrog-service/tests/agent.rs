#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The agent's connections, one question to a model, and what the agent reads and writes.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use studyfrog_agent::exam::{ExamResult, Verdict};
use studyfrog_agent::provider::{HttpRequest, HttpResponse, Message, Protocol, Provider, Role};
use studyfrog_agent::wiki::Change;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::agent::{Ask, ask, load_connections, save_providers, set_key};
use studyfrog_service::agent_vault::{apply_changes, exam_items, record_exam, wiki_context};
use studyfrog_service::create::IdSource;
use studyfrog_service::session::open_vault;

static SEEN: Mutex<Vec<HttpRequest>> = Mutex::new(Vec::new());

fn fake(req: &HttpRequest) -> Result<HttpResponse, String> {
    SEEN.lock().unwrap().push(req.clone());
    Ok(HttpResponse {
        status: 200,
        body: r#"{"choices":[{"message":{"content":"pong"}}]}"#.into(),
    })
}

fn scratch(label: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("studyfrog-agent-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn remote(allowed: bool) -> Provider {
    Provider {
        name: "Cloud".into(),
        protocol: Protocol::OpenAiCompatible,
        base_url: "https://api.example.com".into(),
        model: "m".into(),
        allowed,
    }
}

fn hi() -> Vec<Message> {
    vec![Message {
        role: Role::User,
        content: "ping".into(),
    }]
}

#[test]
fn connections_and_keys_are_stored_privately_and_a_remote_model_waits_for_permission() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("conn");
    save_providers(&dir, &[remote(false)]).unwrap();
    set_key(&dir, "Cloud", Some(" sk-secret ")).unwrap();
    let c = load_connections(&dir);
    assert_eq!(
        (c.providers.len(), c.with_key.contains("Cloud"), c.error),
        (1, true, None)
    );
    assert_eq!(
        fs::metadata(dir.join("secrets.toml"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(
        !fs::read_to_string(dir.join("providers.toml"))
            .unwrap()
            .contains("sk-secret"),
        "keys are not in providers.toml"
    );
    assert!(
        save_providers(&dir, &[remote(false), remote(true)]).is_err(),
        "names are unique"
    );

    let ctx = Ask {
        config_dir: &dir,
        log_dir: Some(&dir),
        now_ms: 1_800_000_000_000,
    };
    let before = SEEN.lock().unwrap().len();
    let refused = ask(fake, &remote(false), ctx, "sys", &hi()).unwrap_err();
    assert!(refused.contains("may not be sent"), "{refused}");
    assert_eq!(SEEN.lock().unwrap().len(), before, "nothing was sent");
    assert_eq!(ask(fake, &remote(true), ctx, "sys", &hi()).unwrap(), "pong");
    let sent = SEEN.lock().unwrap().last().cloned().unwrap();
    assert!(
        sent.headers
            .contains(&("Authorization".into(), "Bearer sk-secret".into()))
    );
    let log = fs::read_to_string(dir.join("agent.log")).unwrap();
    assert!(
        log.contains("Cloud api.example.com 200")
            && !log.contains("ping")
            && !log.contains("sk-secret"),
        "{log}"
    );

    set_key(&dir, "Cloud", None).unwrap();
    assert!(
        ask(fake, &remote(true), ctx, "sys", &hi())
            .unwrap_err()
            .contains("needs an API key")
    );
    fs::write(dir.join("secrets.toml"), "oops = 1").unwrap();
    assert!(
        load_connections(&dir)
            .error
            .unwrap()
            .contains("secrets.toml is not used")
    );
    let _ = fs::remove_dir_all(&dir);
}

fn write(dir: &std::path::Path, rel: &str, n: u128, kind: &str, body: &str) {
    let text = format!(
        "---\nid: {}\nkind: {kind}\ntitle: T{n}\n---\n{body}",
        format_ulid(Uid(n))
    );
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn vault(label: &str) -> (PathBuf, studyfrog_service::session::VaultSession) {
    let dir = scratch(label);
    write(&dir, "a/_stack.md", 1, "stack", "");
    write(&dir, "a/n.md", 2, "note", "About [[T3]].\n");
    write(&dir, "a/c.md", 3, "card", "front?\n???\nback\n");
    write(&dir, "a/q.md", 4, "question", "Prompt?\n???\nReference\n");
    write(&dir, "z/_stack.md", 5, "stack", "");
    write(&dir, "z/n.md", 6, "note", "other");
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (dir, s)
}

#[test]
fn the_context_holds_only_the_chosen_stacks_and_respects_the_budget() {
    let (dir, s) = vault("ctx");
    let c = wiki_context(&s, &["a".to_owned()]).unwrap();
    assert_eq!(c.stacks, [("a".to_owned(), "T1".to_owned())]);
    assert_eq!(c.items.len(), 3, "the other stack is not included");
    assert!(c.items.iter().any(|i| i.kind == "card"
        && i.text.contains("Front: front?")
        && i.text.contains("Back: back")));
    assert_eq!(c.left_out, 0);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn accepted_changes_are_applied_one_by_one_and_failures_do_not_stop_the_rest() {
    let (dir, mut s) = vault("apply");
    let ids = IdSource {
        now_ms: 1_800_000_000_000,
        random: [4; 10],
    };
    let changes = vec![
        Change::CreateNote {
            stack: "a".into(),
            title: "Index".into(),
            body: "- [[T3]]".into(),
        },
        Change::CreateCard {
            stack: "a".into(),
            front: "Capital of Peru?".into(),
            back: "Lima".into(),
            source: None,
        },
        Change::CreateQuestion {
            stack: "z".into(),
            prompt: "Why?".into(),
            reference: "Because".into(),
            source: None,
        },
        Change::UpdateItem {
            uid: format_ulid(Uid(2)),
            fields: vec![("body".into(), "New body".into())],
        },
        Change::UpdateItem {
            uid: format_ulid(Uid(3)),
            fields: vec![("body".into(), "nope".into())],
        },
        Change::UpdateItem {
            uid: "NOT-AN-ID".into(),
            fields: vec![("body".into(), "x".into())],
        },
    ];
    let done = apply_changes(&mut s, &changes, ids, UnixMs(1_800_000_000_000));
    assert_eq!(
        (done.created, done.updated, done.failed.len()),
        (3, 1, 2),
        "{:?}",
        done.failed
    );
    assert!(
        fs::read_to_string(dir.join("a/n.md"))
            .unwrap()
            .contains("New body")
    );
    assert!(
        dir.join("a/index.md").is_file()
            && fs::read_to_string(dir.join("a/capital-of-peru.md"))
                .unwrap()
                .contains("Lima")
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn an_exam_covers_cards_and_questions_and_its_verdicts_become_reviews() {
    let (dir, mut s) = vault("exam");
    let items = exam_items(&s, &["a".to_owned()], 10).unwrap();
    assert_eq!(
        items.iter().map(|i| i.prompt.as_str()).collect::<Vec<_>>(),
        ["front?", "Prompt?"],
        "notes are not examined"
    );
    assert_eq!(exam_items(&s, &["a".to_owned()], 1).unwrap().len(), 1);
    let results = vec![
        ExamResult {
            uid: format_ulid(Uid(3)),
            verdict: Verdict::Correct,
        },
        ExamResult {
            uid: format_ulid(Uid(4)),
            verdict: Verdict::Incorrect,
        },
    ];
    assert_eq!(
        record_exam(&mut s, &results, UnixMs(1_800_000_000_000)).unwrap(),
        2
    );
    let rows = studyfrog_store::reports::load_review_history(&s.conn).unwrap();
    let ratings: Vec<u8> = rows
        .iter()
        .map(|r| studyfrog_core::review::rating_value(r.rating))
        .collect();
    assert_eq!(ratings, [3, 1], "correct is Good, incorrect is Again");
    let _ = fs::remove_dir_all(&dir);
}
