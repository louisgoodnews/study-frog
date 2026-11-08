#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The MCP router: methods, error codes and the policy gate.

mod mcp_common;

use mcp_common::*;

#[test]
fn initialize_names_the_protocol_the_capabilities_and_the_server() {
    let r = call("initialize", &json!({"protocolVersion": "2025-06-18"}));
    assert_eq!(r["id"], 1);
    assert_eq!(r["result"]["protocolVersion"], "2025-06-18");
    for cap in ["tools", "resources", "prompts"] {
        assert!(r["result"]["capabilities"].get(cap).is_some(), "{cap}");
    }
    assert_eq!(
        r["result"]["serverInfo"],
        json!({"name": "test-server", "version": "1.2.3"})
    );
    assert_eq!(call("ping", &Value::Null)["result"], json!({}));
}

#[test]
fn a_notification_gets_no_answer_and_the_id_keeps_its_type() {
    let note = json!({"jsonrpc":"2.0","method":"notifications/initialized"});
    assert!(
        handle_line(
            &registry(),
            &McpPolicy::default(),
            &mut Ctx::default(),
            &note.to_string()
        )
        .is_none()
    );
    let by_text = ask(
        McpPolicy::default(),
        &mut Ctx::default(),
        &json!({"jsonrpc":"2.0","id":"abc","method":"ping"}),
    );
    assert_eq!(by_text["id"], "abc");
}

#[test]
fn broken_messages_get_the_standard_error_codes() {
    let mut ctx = Ctx::default();
    let bad = handle_line(&registry(), &McpPolicy::default(), &mut ctx, "{not json").unwrap();
    let v: Value = serde_json::from_str(&bad).unwrap();
    assert_eq!(
        (v["error"]["code"].as_i64(), &v["id"]),
        (Some(-32700), &Value::Null)
    );
    for request in [
        json!([1, 2]),
        json!({"id": 7, "method": "ping"}),
        json!({"jsonrpc":"1.0","id":7,"method":"ping"}),
        json!({"jsonrpc":"2.0","id":7}),
    ] {
        let v = ask(McpPolicy::default(), &mut ctx, &request);
        assert_eq!(v["error"]["code"], -32600, "{request}");
    }
    let unknown = call("tools/explode", &Value::Null);
    assert_eq!(
        (unknown["error"]["code"].as_i64(), &unknown["id"]),
        (Some(-32601), &json!(1))
    );
}

#[test]
fn the_policy_decides_which_tools_are_listed_and_which_can_run() {
    let list = |policy| {
        names(
            &ask(
                policy,
                &mut Ctx::default(),
                &json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
            ),
            "tools",
        )
    };
    assert_eq!(list(McpPolicy::default()), ["echo", "broken"]);
    assert_eq!(
        list(McpPolicy {
            allow_write: true,
            allow_destructive: false
        }),
        ["echo", "change", "broken"]
    );
    assert_eq!(
        list(McpPolicy {
            allow_write: false,
            allow_destructive: true
        }),
        ["echo", "broken"],
        "destructive needs write too"
    );
    assert_eq!(
        list(McpPolicy {
            allow_write: true,
            allow_destructive: true
        })
        .len(),
        4
    );

    let mut ctx = Ctx::default();
    let call_change = |policy, ctx: &mut Ctx, name: &str| {
        ask(
            policy,
            ctx,
            &json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name}}),
        )
    };
    let refused = call_change(McpPolicy::default(), &mut ctx, "change");
    assert_eq!(refused["result"]["isError"], true);
    assert_eq!(ctx.changes, 0, "a refused tool does not run");
    let wipe = call_change(
        McpPolicy {
            allow_write: true,
            allow_destructive: false,
        },
        &mut ctx,
        "wipe",
    );
    assert_eq!(wipe["result"]["isError"], true);
    assert_eq!(ctx.changes, 0);
    let ran = call_change(
        McpPolicy {
            allow_write: true,
            allow_destructive: false,
        },
        &mut ctx,
        "change",
    );
    assert_eq!(ran["result"]["content"][0]["text"], "changes: 1");
    assert_eq!(ran["result"]["isError"], false);
}

#[test]
fn tools_list_carries_the_schema_and_the_annotations() {
    let v = call("tools/list", &Value::Null);
    let echo = &v["result"]["tools"][0];
    assert_eq!(echo["inputSchema"]["type"], "object");
    assert_eq!(echo["annotations"]["readOnlyHint"], true);
    assert_eq!(echo["annotations"]["idempotentHint"], true);
    assert_eq!(echo["annotations"]["destructiveHint"], false);
    assert_eq!(echo["annotations"]["openWorldHint"], false);
}
