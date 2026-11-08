#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The MCP router: tools, resources, prompts, registration and addresses.

mod mcp_common;

use mcp_common::*;

#[test]
fn calling_a_tool_checks_its_name_and_arguments_and_reports_its_failures_as_content() {
    let ok = call(
        "tools/call",
        &json!({"name": "echo", "arguments": {"text": "hi"}}),
    );
    assert_eq!(ok["result"]["content"][0]["text"], "hi");
    assert_eq!(
        call("tools/call", &json!({"name": "nope"}))["error"]["code"],
        -32602
    );
    assert_eq!(call("tools/call", &json!({}))["error"]["code"], -32602);
    assert_eq!(
        call("tools/call", &json!({"name": "echo", "arguments": [1]}))["error"]["code"],
        -32602
    );
    let failed = call("tools/call", &json!({"name": "broken"}));
    assert_eq!(failed["result"]["isError"], true);
    assert_eq!(
        failed["result"]["content"][0]["text"],
        "the vault is closed"
    );
}

#[test]
fn resources_are_listed_as_fixed_or_templated_and_read_by_address() {
    assert_eq!(
        names(&call("resources/list", &Value::Null), "resources"),
        ["Summary"]
    );
    let templates = call("resources/templates/list", &Value::Null);
    assert_eq!(
        templates["result"]["resourceTemplates"][0]["uriTemplate"],
        "test://items/{id}"
    );
    let fixed = call("resources/read", &json!({"uri": "test://summary"}));
    assert_eq!(fixed["result"]["contents"][0]["text"], "item none");
    let item = call("resources/read", &json!({"uri": "test://items/42"}));
    assert_eq!(item["result"]["contents"][0]["text"], "item 42");
    assert_eq!(item["result"]["contents"][0]["mimeType"], "text/markdown");
    assert_eq!(
        call("resources/read", &json!({"uri": "test://items/"}))["error"]["code"],
        RESOURCE_NOT_FOUND
    );
    assert_eq!(
        call("resources/read", &json!({"uri": "other://x"}))["error"]["code"],
        RESOURCE_NOT_FOUND
    );
    assert_eq!(call("resources/read", &json!({}))["error"]["code"], -32602);
}

#[test]
fn prompts_are_listed_with_their_arguments_and_need_the_required_ones() {
    let list = call("prompts/list", &Value::Null);
    assert_eq!(
        list["result"]["prompts"][0]["arguments"][0]["required"],
        true
    );
    let got = call(
        "prompts/get",
        &json!({"name": "greet", "arguments": {"name": "Ada"}}),
    );
    assert_eq!(got["result"]["messages"][0]["content"]["text"], "Hello Ada");
    assert_eq!(got["result"]["messages"][0]["role"], "user");
    assert_eq!(
        call("prompts/get", &json!({"name": "greet"}))["error"]["code"],
        -32602
    );
    assert_eq!(
        call("prompts/get", &json!({"name": "nope"}))["error"]["code"],
        -32602
    );
}

#[test]
fn registering_again_replaces_instead_of_duplicating() {
    let reg = register_tool(registry(), tool("echo", WRITE, change));
    assert_eq!(reg.tools.iter().filter(|t| t.name == "echo").count(), 1);
    assert!(
        !reg.tools
            .iter()
            .find(|t| t.name == "echo")
            .unwrap()
            .annotations
            .read_only_hint
    );
    assert_eq!(reg.resources.len(), 2);
}

#[test]
fn addresses_match_their_templates_part_by_part() {
    let one = |t, u| match_template(t, u);
    assert_eq!(
        one("a://items/{id}", "a://items/7"),
        Some(vec![("id".to_owned(), "7".to_owned())])
    );
    assert_eq!(
        one("a://s/{stack}/i/{id}", "a://s/fr/i/9"),
        Some(vec![
            ("stack".to_owned(), "fr".to_owned()),
            ("id".to_owned(), "9".to_owned())
        ])
    );
    assert_eq!(one("a://summary", "a://summary"), Some(Vec::new()));
    assert_eq!(one("a://summary", "a://summary/x"), None);
    assert_eq!(
        one("a://items/{id}", "a://items/"),
        None,
        "a part is not empty"
    );
    assert_eq!(
        one("a://items/{id}", "a://items/1/2"),
        None,
        "a part has no slash"
    );
    assert_eq!(one("a://items/{id}", "b://items/1"), None);
}
