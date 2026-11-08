#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The MCP structural tools need both permissions and keep undo entries that restore everything,
//! and the standard prompts are MCP prompts with their arguments.

mod common;
mod mcp_common;

use mcp_common::*;
use studyfrog_service::mcp_undo::{list_undo, undo_recorded};

const ALL: McpPolicy = McpPolicy {
    allow_write: true,
    allow_destructive: true,
};

#[test]
fn deleting_needs_both_permissions_and_is_undone_completely() {
    let vault = sample_vault("mcp-delete");
    let before = markdown_files(&vault.path);
    let mut c = ctx(&vault.path);
    c.session.conn.execute(
        "INSERT INTO mem_state (item_uid, stack_uid, stability, difficulty, due_ms, last_review_ms) VALUES (?1, ?2, 3.5, 6.0, 1, 1)",
        [format_ulid(Uid(10)), format_ulid(Uid(1))],
    ).unwrap();
    let args = json!({"item": format_ulid(Uid(10))});
    for policy in [
        McpPolicy::default(),
        McpPolicy {
            allow_write: true,
            allow_destructive: false,
        },
    ] {
        let refused = call(&mut c, policy, "vault_delete_item", args.clone());
        assert!(
            refused.as_str().unwrap().contains("is not allowed"),
            "{refused}"
        );
    }
    assert_eq!(markdown_files(&vault.path), before);
    let done = call(&mut c, ALL, "vault_delete_item", args);
    assert_eq!(done["deleted"], "french/a.md", "{done}");
    assert!(!vault.path.join("french/a.md").exists());
    let undo = list_undo(&vault.path);
    assert_eq!(undo.len(), 1);
    assert!(undo[0].label.contains("Deleted \"A\""));
    undo_recorded(&mut c.session, &undo[0].id).unwrap();
    assert_eq!(
        markdown_files(&vault.path),
        before,
        "the file is back byte for byte"
    );
    assert!(list_undo(&vault.path).is_empty());
}

#[test]
fn moving_a_stack_keeps_schedules_and_its_undo_entry_survives_a_restart() {
    let vault = sample_vault("mcp-move");
    std::fs::create_dir_all(vault.path.join("latin")).unwrap();
    std::fs::write(
        vault.path.join("latin/_stack.md"),
        format!(
            "---\nid: {}\nkind: stack\ntitle: Latin\n---\n",
            format_ulid(Uid(2))
        ),
    )
    .unwrap();
    let before = markdown_files(&vault.path);
    let mut c = ctx(&vault.path);
    let moved = call(
        &mut c,
        ALL,
        "vault_move_stack",
        json!({"stack": "Latin", "into": "French"}),
    );
    assert_eq!(moved["to"], "french/latin", "{moved}");
    assert_eq!(moved["schedules_kept"], true);
    let missing = call(&mut c, ALL, "vault_move_stack", json!({"stack": "Greek"}));
    assert!(missing.as_str().unwrap().contains("no stack Greek"));
    drop(c);
    let mut c = ctx(&vault.path);
    let undo = list_undo(&vault.path);
    assert_eq!(undo.len(), 1, "read back from its file");
    undo_recorded(&mut c.session, &undo[0].id).unwrap();
    assert_eq!(markdown_files(&vault.path), before);
}

#[test]
fn every_standard_prompt_is_an_mcp_prompt_with_its_arguments() {
    let vault = sample_vault("mcp-prompts");
    let mut c = ctx(&vault.path);
    let list = session(
        &mut c,
        McpPolicy::default(),
        &[request(1, "prompts/list", &Value::Null)],
    );
    let prompts = list[0]["result"]["prompts"].as_array().unwrap().clone();
    assert_eq!(prompts.len(), studyfrog_service::prompts::PROMPTS.len());
    let explain = prompts.iter().find(|p| p["name"] == "explain").unwrap();
    let args: Vec<&str> = explain["arguments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();
    assert_eq!(args, ["file_path", "target_stack", "depth"]);
    assert!(!explain["description"].as_str().unwrap().is_empty());
    let get = |c: &mut VaultCtx, name: &str, arguments: Value| {
        let m = request(
            1,
            "prompts/get",
            &json!({"name": name, "arguments": arguments}),
        );
        session(c, McpPolicy::default(), &[m]).remove(0)
    };
    let explained = get(
        &mut c,
        "explain",
        json!({"depth": "technical", "file_path": "french/c.md", "target_stack": "French"}),
    );
    let text = explained["result"]["messages"][0]["content"]["text"]
        .as_str()
        .unwrap();
    assert!(
        text.contains("the technical level only"),
        "the depth is put in"
    );
    assert!(text.contains("The attached file is `french/c.md`"));
    assert!(text.contains("into the stack `french`"));
    let exam = get(&mut c, "exam", json!({"count": "3"}));
    let text = exam["result"]["messages"][0]["content"]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains('3'));
    let outside = get(&mut c, "flashcards", json!({"file_path": "../secret.md"}));
    assert!(
        outside["error"]["message"]
            .as_str()
            .unwrap()
            .contains("not a file in the vault")
    );
}
