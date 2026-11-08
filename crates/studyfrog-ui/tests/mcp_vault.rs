#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Model Context Protocol server on a real vault: a whole session, the search and that nothing changes.

mod common;
mod mcp_common;

use mcp_common::*;

#[test]
fn a_whole_session_initializes_lists_searches_and_reads() {
    let vault = sample_vault("mcp-session");
    let mut c = ctx(&vault.path);
    let messages = [
        request(1, "initialize", &json!({"protocolVersion": "2025-06-18"})),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        request(2, "tools/list", &Value::Null),
        request(
            3,
            "resources/read",
            &json!({"uri": "studyfrog://vault/summary"}),
        ),
        request(4, "ping", &Value::Null),
    ];
    let answers = session(&mut c, McpPolicy::default(), &messages);
    assert_eq!(answers.len(), 4, "the notification gets no answer");
    assert_eq!(
        answers[0]["result"]["serverInfo"],
        json!({"name": "studyfrog", "version": "9.9.9"})
    );
    let tools = answers[1]["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(
        names[..5],
        READ_ONLY_TOOLS,
        "then the four staging authoring tools"
    );
    assert_eq!(names.len(), 9, "no structural tool without permission");
    for t in &tools[..5] {
        assert_eq!(t["annotations"]["readOnlyHint"], true);
        assert_eq!(t["annotations"]["idempotentHint"], true);
        assert_eq!(t["annotations"]["destructiveHint"], false);
    }
    let summary: Value = serde_json::from_str(
        answers[2]["result"]["contents"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let french = &summary["stacks"][0];
    assert_eq!(
        (
            french["title"].as_str(),
            french["cards"].as_u64(),
            french["notes"].as_u64()
        ),
        (Some("French"), Some(2), Some(1))
    );
    assert_eq!(summary["items_outside_stacks"], 1);
    assert_eq!(answers[3]["result"], json!({}));
}

#[test]
fn the_search_understands_queries_plain_words_and_reports_bad_ones() {
    let vault = sample_vault("mcp-search");
    let mut c = ctx(&vault.path);
    assert_eq!(titles(&search(&mut c, "type = card")), ["A", "B"]);
    assert_eq!(titles(&search(&mut c, "stack = French")), ["A", "B", "C"]);
    assert_eq!(
        titles(&search(&mut c, "Loose")),
        ["Loose"],
        "plain words are a text search"
    );
    let limited =
        json!({"name": "vault_search_items", "arguments": {"query": "type = card", "limit": 1}});
    let one = session(
        &mut c,
        McpPolicy::default(),
        &[request(1, "tools/call", &limited)],
    )
    .remove(0);
    assert_eq!(found(&one)["count"], 1);
    let bad = search(&mut c, "type = ");
    assert_eq!(bad["result"]["isError"], true);
    assert!(
        bad["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("column")
    );
    let missing = session(
        &mut c,
        McpPolicy::default(),
        &[request(
            1,
            "tools/call",
            &json!({"name": "vault_search_items", "arguments": {}}),
        )],
    )
    .remove(0);
    assert_eq!(
        missing["result"]["isError"], true,
        "a tool's failure is content"
    );
    let why = missing["result"]["content"][0]["text"].as_str().unwrap();
    assert!(why.contains("query"), "{why}");
}

#[test]
fn items_and_stacks_are_read_by_id_and_bad_ids_are_refused() {
    let vault = sample_vault("mcp-read");
    let mut c = ctx(&vault.path);
    let read = |c: &mut VaultCtx, uri: &str| {
        session(
            c,
            McpPolicy::default(),
            &[request(1, "resources/read", &json!({"uri": uri}))],
        )
        .remove(0)
    };
    let item = read(
        &mut c,
        &format!("studyfrog://items/{}", format_ulid(Uid(10))),
    );
    assert_eq!(item["result"]["contents"][0]["mimeType"], "text/markdown");
    let text = item["result"]["contents"][0]["text"].as_str().unwrap();
    assert!(text.contains("title: A") && text.contains("???"), "{text}");
    let stack = read(
        &mut c,
        &format!("studyfrog://stacks/{}", format_ulid(Uid(1))),
    );
    let stack: Value =
        serde_json::from_str(stack["result"]["contents"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(stack["title"], "French");
    assert_eq!(stack["items"].as_array().unwrap().len(), 3);
    assert_eq!(
        read(
            &mut c,
            &format!("studyfrog://items/{}", format_ulid(Uid(999)))
        )["error"]["code"],
        -32002
    );
    assert_eq!(
        read(
            &mut c,
            &format!("studyfrog://stacks/{}", format_ulid(Uid(999)))
        )["error"]["code"],
        -32002
    );
    assert_eq!(
        read(&mut c, "studyfrog://items/not-an-id")["error"]["code"],
        -32002,
        "not an id, and no item has it as its key"
    );
    assert_eq!(
        read(&mut c, "studyfrog://diary/not-a-date")["error"]["code"],
        -32602
    );
    assert_eq!(
        read(&mut c, "studyfrog://nothing/here")["error"]["code"],
        -32002
    );
}
