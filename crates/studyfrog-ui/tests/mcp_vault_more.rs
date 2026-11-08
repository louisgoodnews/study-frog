#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Model Context Protocol server on a real vault: reading by id, the lines, the flags and the config block.

mod common;
mod mcp_common;

use mcp_common::*;

#[test]
fn a_session_changes_nothing_in_the_vault_even_when_writing_is_allowed() {
    let vault = sample_vault("mcp-readonly");
    let before = markdown_files(&vault.path);
    let mut c = ctx(&vault.path);
    let policy = McpPolicy {
        allow_write: true,
        allow_destructive: true,
    };
    let id = format_ulid(Uid(10));
    let messages = [
        request(1, "tools/list", &Value::Null),
        request(
            2,
            "tools/call",
            &json!({"name": "vault_search_items", "arguments": {"query": "type = card"}}),
        ),
        request(
            3,
            "resources/read",
            &json!({"uri": "studyfrog://vault/summary"}),
        ),
        request(
            4,
            "resources/read",
            &json!({"uri": format!("studyfrog://items/{id}")}),
        ),
        request(5, "tools/call", &json!({"name": "vault_delete_everything"})),
    ];
    let answers = session(&mut c, policy, &messages);
    assert_eq!(
        answers[0]["result"]["tools"].as_array().unwrap().len(),
        1,
        "no write tools exist yet"
    );
    assert_eq!(answers[4]["error"]["code"], -32602);
    assert_eq!(markdown_files(&vault.path), before);
}

#[test]
fn every_line_written_is_a_json_rpc_message_and_blank_lines_are_skipped() {
    let vault = sample_vault("mcp-lines");
    let mut c = ctx(&vault.path);
    let input = format!(
        "\n   \n{}\nnot json\n{}\n",
        request(1, "ping", &Value::Null),
        request(2, "ping", &Value::Null)
    );
    let mut out = Vec::new();
    serve_lines(
        &vault_registry("1"),
        &McpPolicy::default(),
        &mut c,
        Cursor::new(input),
        &mut out,
    )
    .unwrap();
    let text = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);
    for line in &lines {
        let v: Value = serde_json::from_str(line).unwrap();
        assert_eq!(v["jsonrpc"], "2.0", "{line}");
    }
    assert!(
        lines[1].contains("-32700"),
        "the broken line is answered with a parse error"
    );
    assert!(text.ends_with('\n'));
}

#[test]
fn the_flags_choose_the_vault_and_the_policy() {
    let args = |list: &[&str]| list.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        parse_mcp_args(&args(&["--mcp"])).unwrap(),
        McpArgs::default()
    );
    let all = parse_mcp_args(&args(&[
        "--mcp",
        "--vault",
        "/v",
        "--mcp-allow-write",
        "--mcp-allow-destructive",
    ]))
    .unwrap();
    assert_eq!(all.vault.as_deref(), Some("/v"));
    assert_eq!(
        all.policy,
        McpPolicy {
            allow_write: true,
            allow_destructive: true
        }
    );
    assert!(
        parse_mcp_args(&args(&["--mcp", "--vault"]))
            .unwrap_err()
            .contains("folder")
    );
    assert!(
        parse_mcp_args(&args(&["--mcp", "--loud"]))
            .unwrap_err()
            .contains("--loud")
    );
}

#[test]
fn the_config_block_starts_the_server_for_the_vault() {
    let text = mcp_config_snippet("/opt/studyfrog", "/home/me/my notes");
    let v: Value = serde_json::from_str(&text).unwrap();
    let server = &v["mcpServers"]["studyfrog"];
    assert_eq!(server["command"], "/opt/studyfrog");
    assert_eq!(
        server["args"],
        json!(["--mcp", "--vault", "/home/me/my notes"])
    );
}
