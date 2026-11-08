#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! A whole wiki session through the worker with a fake model.

use std::sync::Mutex;

use studyfrog_agent::provider::{HttpRequest, HttpResponse, Protocol, Provider};
use studyfrog_ui::agent_types::AgentMsg;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_with_host};
use studyfrog_ui::host::Host;
use studyfrog_ui::msg::Msg;

fn local() -> Provider {
    Provider {
        name: "Local".into(),
        protocol: Protocol::Ollama,
        base_url: "http://localhost:11434".into(),
        model: "llama".into(),
        allowed: false,
    }
}

static ASKED: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn fake_model(req: &HttpRequest) -> Result<HttpResponse, String> {
    ASKED.lock().unwrap().push(req.body.clone());
    let reply = r#"{"summary":"Made an index","changes":[{"action":"create_note","stack":"deck","title":"Index","body":"- [[C1]]"}]}"#;
    let escaped = reply.replace('\\', "\\\\").replace('"', "\\\"");
    let body = format!("{{\"message\":{{\"role\":\"assistant\",\"content\":\"{escaped}\"}}}}");
    Ok(HttpResponse {
        headers: Vec::new(),
        status: 200,
        body,
    })
}

#[test]
fn the_worker_runs_a_whole_wiki_session_with_a_fake_model() {
    let base = std::env::temp_dir().join(format!("studyfrog-agent-ui-{}", std::process::id()));
    let (vault, cfg) = (base.join("vault"), base.join("cfg"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(vault.join("deck")).unwrap();
    std::fs::write(
        vault.join("deck/_stack.md"),
        "---\nid: 0000000000000000000000000S\nkind: stack\ntitle: Deck\n---\n",
    )
    .unwrap();
    std::fs::write(
        vault.join("deck/c1.md"),
        "---\nid: 0000000000000000000000000C\nkind: card\ntitle: C1\n---\nfront?\n???\nback\n",
    )
    .unwrap();
    let host = Host {
        transport: Some(fake_model),
        ..Host::default()
    };
    let mut w = worker_with_host(Some(cfg.clone()), None, host);
    let now = 1_800_000_000_000_000_000;
    execute(&mut w, Effect::OpenVault(vault.display().to_string()), now);

    let saved = execute(
        &mut w,
        Effect::SaveConnection {
            provider: local(),
            key: None,
        },
        now,
    );
    assert!(matches!(&saved[0], Msg::Agent(AgentMsg::Saved(Ok(c))) if c.providers.len() == 1));
    let prepared = execute(
        &mut w,
        Effect::PrepareWiki {
            folders: vec!["deck".into()],
            pdf: None,
        },
        now,
    );
    let Msg::Agent(AgentMsg::WikiPrepared(Ok(ctx))) = &prepared[0] else {
        panic!("{:?}", prepared[0])
    };
    assert_eq!(ctx.items.len(), 1);
    let uids: Vec<String> = ctx.items.iter().map(|i| i.uid.clone()).collect();
    let chat = execute(
        &mut w,
        Effect::AgentChat {
            provider: local(),
            system: "s".into(),
            messages: vec![studyfrog_agent::provider::Message {
                role: studyfrog_agent::provider::Role::User,
                content: "go".into(),
            }],
            purpose: studyfrog_ui::agent_types::ChatPurpose::Wiki(uids, vec!["deck".into()]),
        },
        now,
    );
    let Msg::Agent(AgentMsg::WikiReplied(Ok(p))) = &chat[0] else {
        panic!("{:?}", chat[0])
    };
    assert_eq!((p.summary.as_str(), p.changes.len()), ("Made an index", 1));
    let done = execute(&mut w, Effect::ApplyChanges(p.changes.clone()), now);
    let Msg::Agent(AgentMsg::ProposalApplied(Ok(r))) = &done[0] else {
        panic!("{:?}", done[0])
    };
    assert_eq!(r.0.created, 1);
    assert!(vault.join("deck/index.md").is_file());
    assert!(
        ASKED.lock().unwrap().iter().all(|b| !b.contains("sk-")),
        "no key in the request body"
    );
    let _ = std::fs::remove_dir_all(&base);
}
