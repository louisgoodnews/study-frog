#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The MCP authoring tools: they stage by default, write only when allowed, and what they staged is
//! applied by the same service as the assistant's proposals.

mod common;
mod mcp_common;

use mcp_common::*;
use studyfrog_service::agent_vault::apply_changes;
use studyfrog_service::create::IdSource;
use studyfrog_service::mcp_staging::{list_staged, remove_staged};

const ALLOW_WRITE: McpPolicy = McpPolicy {
    allow_write: true,
    allow_destructive: false,
};

fn ids() -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000,
        random: [3; 10],
    }
}

fn names(c: &mut VaultCtx, policy: McpPolicy) -> Vec<String> {
    let answer = session(c, policy, &[request(1, "tools/list", &Value::Null)]).remove(0);
    answer["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn authoring_tools_are_listed_without_permission_but_structural_ones_are_not() {
    let vault = sample_vault("mcp-list-authoring");
    let mut c = ctx(&vault.path);
    let default = names(&mut c, McpPolicy::default());
    for staged in [
        "vault_propose_items",
        "note_extract_inline_cards",
        "diary_log_entry",
        "calendar_plan_session",
    ] {
        assert!(default.contains(&staged.to_owned()), "{staged}");
    }
    assert!(!default.contains(&"vault_delete_item".to_owned()));
    assert!(!names(&mut c, ALLOW_WRITE).contains(&"vault_move_stack".to_owned()));
    let all = McpPolicy {
        allow_write: true,
        allow_destructive: true,
    };
    let listed = session(&mut c, all, &[request(1, "tools/list", &Value::Null)]).remove(0);
    let delete = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "vault_delete_item")
        .unwrap()
        .clone();
    assert_eq!(delete["annotations"]["destructiveHint"], true);
    assert_eq!(delete["annotations"]["readOnlyHint"], false);
}

#[test]
fn proposed_items_are_staged_checked_and_written_only_when_applied() {
    let vault = sample_vault("mcp-propose");
    let mut c = ctx(&vault.path);
    let before = markdown_files(&vault.path);
    let answer = call(
        &mut c,
        McpPolicy::default(),
        "vault_propose_items",
        json!({"summary": "Greetings", "items": [
            {"kind": "card", "stack": "French", "front": "Hello", "back": "Bonjour\\nor Salut"},
            {"kind": "question", "stack": "french", "prompt": "Say goodbye", "reference": "Au revoir"},
            {"kind": "card", "stack": "Latin", "front": "x", "back": "y"},
        ], "csv": "stack,kind,front,back\nfrench,card,Thanks,Merci\n"}),
    );
    assert_eq!(answer["changes"], 3, "{answer}");
    assert_eq!(
        answer["rejected"].as_array().unwrap().len(),
        1,
        "Latin is no stack"
    );
    assert_eq!(markdown_files(&vault.path), before, "nothing written yet");
    let staged = list_staged(&c.session).unwrap();
    assert_eq!(staged.len(), 1);
    let p = &staged[0].proposal;
    assert_eq!(
        (p.tool.as_str(), p.summary.as_str()),
        ("vault_propose_items", "Greetings")
    );
    let done = apply_changes(
        &mut c.session,
        &p.changes,
        ids(),
        studyfrog_core::time::UnixMs(now()),
    );
    assert_eq!((done.created, done.failed.len()), (3, 0), "{done:?}");
    remove_staged(&c.session.root, &p.id).unwrap();
    assert!(list_staged(&c.session).unwrap().is_empty());
    let after = markdown_files(&vault.path);
    assert_eq!(after.len(), before.len() + 3);
    let card = after
        .iter()
        .find(|(_, t)| String::from_utf8_lossy(t).contains("Bonjour"))
        .unwrap();
    assert!(
        String::from_utf8_lossy(card.1).contains("Bonjour\nor Salut"),
        "new line written out"
    );
}

#[test]
fn with_permission_to_write_items_are_written_directly() {
    let vault = sample_vault("mcp-propose-write");
    let mut c = ctx(&vault.path);
    let answer = call(
        &mut c,
        ALLOW_WRITE,
        "vault_propose_items",
        json!({"items": [{"kind": "note", "stack": "French", "title": "Verbs", "body": "Être and avoir."}]}),
    );
    assert_eq!(answer["applied"]["created"], 1, "{answer}");
    assert!(list_staged(&c.session).unwrap().is_empty());
    assert!(vault.path.join("french").read_dir().unwrap().count() > 4);
}

#[test]
fn diary_entries_and_sessions_are_staged_and_then_applied() {
    let vault = sample_vault("mcp-diary-plan");
    let mut c = ctx(&vault.path);
    let p = McpPolicy::default();
    let diary = call(
        &mut c,
        p,
        "diary_log_entry",
        json!({"date": "2026-11-07", "body": "Verbs went well.", "study_minutes": 30, "ratings": {"focus": 4, "understanding": 5}}),
    );
    assert!(diary["staged"].is_string(), "{diary}");
    let bad = call(
        &mut c,
        p,
        "diary_log_entry",
        json!({"date": "2026-11-07", "ratings": {"focus": 9}}),
    );
    assert!(bad.as_str().unwrap().contains("1 to 5"));
    let plan = call(
        &mut c,
        p,
        "calendar_plan_session",
        json!({"title": "French drill", "date": "2026-11-09", "start": "18:30", "duration_minutes": 45, "stack": "French"}),
    );
    assert!(plan["staged"].is_string(), "{plan}");
    let late = call(
        &mut c,
        p,
        "calendar_plan_session",
        json!({"title": "x", "date": "2026-11-09", "start": "25:00", "duration_minutes": 5}),
    );
    assert!(late.as_str().unwrap().contains("HH:MM"));
    assert!(
        !vault.path.join("diary").exists(),
        "staging writes no diary file"
    );
    for staged in list_staged(&c.session).unwrap() {
        let done = apply_changes(
            &mut c.session,
            &staged.proposal.changes,
            ids(),
            studyfrog_core::time::UnixMs(now()),
        );
        assert_eq!(done.failed, Vec::<String>::new());
    }
    let day = content(&read(&mut c, "studyfrog://diary/2026-11-07"), 0);
    assert_eq!(day["entries"][0]["ratings"]["understanding"], 5);
    assert_eq!(day["entries"][0]["study_minutes"], 30);
    let sessions = studyfrog_service::planning::list_sessions(&c.session).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(
        (sessions[0].start.minute, sessions[0].duration_min),
        (18 * 60 + 30, 45)
    );
    assert!(sessions[0].stack.is_some());
}

#[test]
fn inline_cards_of_a_note_are_proposed_once() {
    let vault = sample_vault("mcp-inline");
    std::fs::write(
        vault.path.join("french/c.md"),
        format!(
            "---\nid: {}\nkind: note\ntitle: C\n---\n# Words\n- chat :: cat\n```\nfake :: no\n```\nQ: dog?\nA: chien\n??: Why learn French?\nA: To speak it.\n",
            format_ulid(Uid(12))
        ),
    )
    .unwrap();
    let mut c = ctx(&vault.path);
    let first = call(
        &mut c,
        ALLOW_WRITE,
        "note_extract_inline_cards",
        json!({"note": format_ulid(Uid(12))}),
    );
    assert_eq!(
        first["applied"]["created"], 3,
        "two cards and a question, not the code: {first}"
    );
    let again = call(
        &mut c,
        ALLOW_WRITE,
        "note_extract_inline_cards",
        json!({"note": format_ulid(Uid(12))}),
    );
    assert_eq!(again["already_there"], 3, "{again}");
    let not_note = call(
        &mut c,
        ALLOW_WRITE,
        "note_extract_inline_cards",
        json!({"note": format_ulid(Uid(10))}),
    );
    assert!(not_note.as_str().unwrap().contains("not a note"));
}
