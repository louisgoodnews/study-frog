#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0290: turns through a local and a consented remote provider, refused requests, the prompt budget and reopening the vault.

mod classroom_common;

use classroom_common::*;

use std::cell::RefCell;
use std::fs;
use studyfrog_agent::provider::{HttpRequest, HttpResponse};
use studyfrog_core::classroom::{SourceKind, SourceRef};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::agent::Ask;
use studyfrog_service::classroom_context::{LECTURE_BUDGET_TOKENS, Lecture};
use studyfrog_service::classroom_model::provider_model;
use studyfrog_service::classroom_turns::{advance_segment, simulated_question, start_session};
use studyfrog_service::create::IdSource;
use studyfrog_service::session::open_vault;
use studyfrog_store::reindex::reindex;

fn transport_ok(req: &HttpRequest) -> Result<HttpResponse, String> {
    let text = if req.body.contains("Mitochondria") {
        "grounded reply"
    } else {
        "ungrounded reply"
    };
    let body = if req.url.contains("/api/chat") {
        format!(r#"{{"message":{{"content":"{text}"}}}}"#)
    } else {
        format!(r#"{{"choices":[{{"message":{{"content":"{text}"}}}}]}}"#)
    };
    Ok(HttpResponse {
        status: 200,
        body,
        headers: Vec::new(),
    })
}

fn transport_must_not_run(_: &HttpRequest) -> Result<HttpResponse, String> {
    panic!("a provider without consent must not be contacted")
}

#[test]
fn turns_go_through_a_local_model_and_a_consented_remote_one() {
    let (d, s) = vault("providers");
    let cfg = d.0.join("config");
    studyfrog_service::agent::set_key(&cfg, "cloud", Some("test-key")).unwrap();
    for (seed, provider) in [(3_u8, local()), (9_u8, remote(true))] {
        let ids = IdSource {
            now_ms: 1_000,
            random: [seed; 10],
        };
        let id = start_session(&s, ids, "Cell lecture", &sources(), people()).unwrap();
        let lecture = Lecture {
            vault: &s,
            provider: &provider,
            id,
        };
        let ctx = Ask {
            config_dir: &cfg,
            log_dir: None,
            now_ms: 1,
        };
        let mut model = provider_model(transport_ok, &provider, ctx);
        let reply = advance_segment(&lecture, &mut model, AT).unwrap();
        assert_eq!(reply, "grounded reply", "{}", provider.name);
        assert_eq!(stored(&s, id).turns.len(), 1, "{}", provider.name);
    }
}

#[test]
fn an_unconsented_remote_provider_is_refused_before_any_request() {
    let (d, s) = vault("no-request");
    let cfg = d.0.join("config");
    let id = start(&s);
    let provider = remote(false);
    let lecture = Lecture {
        vault: &s,
        provider: &provider,
        id,
    };
    let ctx = Ask {
        config_dir: &cfg,
        log_dir: None,
        now_ms: 1,
    };
    let mut model = provider_model(transport_must_not_run, &provider, ctx);
    assert!(advance_segment(&lecture, &mut model, AT).is_err());
    assert!(stored(&s, id).turns.is_empty());
}

#[test]
fn a_long_lecture_keeps_every_prompt_within_the_budget() {
    let (d, mut s) = vault("budget");
    let long: String = (0..200)
        .map(|i| format!("# Topic {i}\nFact {i}: {}\n", "word ".repeat(40)))
        .collect();
    fs::write(
        d.0.join("long.md"),
        format!("---\nid: 00000000000000000000000005\nkind: note\ntitle: Long\n---\n{long}"),
    )
    .unwrap();
    reindex(&mut s.conn, &s.root).unwrap();
    let ids = IdSource {
        now_ms: 1_000,
        random: [5; 10],
    };
    let src = [SourceRef {
        kind: SourceKind::Note,
        uid: Uid(5),
    }];
    let id = start_session(&s, ids, "Long", &src, people()).unwrap();
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    for n in 0..4 {
        advance_segment(&lecture, &mut fake(&log), UnixMs(10_000 + n)).unwrap();
    }
    let limit = 3 * LECTURE_BUDGET_TOKENS + 300;
    for prompt in log.borrow().iter() {
        let user = prompt.split_once('\n').map_or("", |(_, u)| u);
        assert!(
            user.chars().count() <= limit,
            "{} chars",
            user.chars().count()
        );
    }
}

#[test]
fn the_stored_session_survives_reopening_the_vault() {
    let (d, s) = vault("reopen");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    advance_segment(&lecture, &mut fake(&log), AT).unwrap();
    simulated_question(&lecture, &mut fake(&log), UnixMs(6_000), 1).unwrap();
    let before = stored(&s, id);
    let (reopened, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    assert_eq!(stored(&reopened, id), before);
    assert_eq!(before.turns.len(), 3);
}
