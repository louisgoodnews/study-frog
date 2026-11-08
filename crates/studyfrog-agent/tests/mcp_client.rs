#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The assistant's MCP client as data: the servers file, consent and approval, and tool calls in a
//! conversation.

use serde_json::json;
use studyfrog_agent::mcp_client::{
    Discovered, McpServer, McpTransport, RemoteTool, needs_approval, server_refusal,
};
use studyfrog_agent::mcp_client_cfg::{parse_servers, render_servers, split_command};
use studyfrog_agent::mcp_client_chat::{parse_tool_call, result_message, tools_block};

fn server(name: &str, transport: McpTransport) -> McpServer {
    McpServer {
        name: name.into(),
        transport,
        enabled: true,
        trust_read_only: false,
        allowed: false,
    }
}

fn tool(name: &str, read_only: bool) -> RemoteTool {
    RemoteTool {
        name: name.into(),
        description: "d".into(),
        input_schema: json!({"type": "object"}),
        read_only,
        destructive: false,
    }
}

#[test]
fn the_servers_file_reads_back_and_bad_entries_are_refused() {
    let (cmd, args) = split_command(r#"npx -y "my server" --flag"#).unwrap();
    assert_eq!(
        (cmd.as_str(), args),
        (
            "npx",
            vec!["-y".to_owned(), "my server".into(), "--flag".into()]
        )
    );
    let mut zotero = server(
        "zotero",
        McpTransport::Stdio {
            command: cmd,
            args: vec!["a \"q\"".into()],
        },
    );
    zotero.trust_read_only = true;
    let web = server(
        "arxiv",
        McpTransport::Http {
            url: "https://x.org/mcp".into(),
        },
    );
    let list = vec![zotero, web];
    assert_eq!(parse_servers(&render_servers(&list)).unwrap(), list);
    for bad in [
        "[[server]]\nname = \"a\"\n",
        "[[server]]\nname = \"a\"\ncommand = \"x\"\nurl = \"http://y\"\n",
        "[[server]]\nname = \"a\"\nurl = \"ftp://y\"\n",
        "[[server]]\nname = \"a/b\"\ncommand = \"x\"\n",
        "[[server]]\nname = \"a\"\ncommand = \"x\"\n[[server]]\nname = \"A\"\ncommand = \"y\"\n",
    ] {
        assert!(parse_servers(bad).is_err(), "{bad}");
    }
    assert!(parse_servers("").unwrap().is_empty());
}

#[test]
fn only_trusted_read_only_tools_run_without_asking_and_remote_servers_need_consent() {
    let mut s = server(
        "z",
        McpTransport::Stdio {
            command: "z".into(),
            args: vec![],
        },
    );
    assert!(needs_approval(&s, &tool("find", true)), "not trusted yet");
    s.trust_read_only = true;
    assert!(!needs_approval(&s, &tool("find", true)));
    assert!(
        needs_approval(&s, &tool("save", false)),
        "changing tools always ask"
    );
    assert_eq!(server_refusal(&s), None);
    s.enabled = false;
    assert!(server_refusal(&s).unwrap().contains("switched off"));
    let mut remote = server(
        "r",
        McpTransport::Http {
            url: "https://r.org".into(),
        },
    );
    assert!(server_refusal(&remote).is_some());
    remote.allowed = true;
    assert_eq!(server_refusal(&remote), None);
    let local = server(
        "l",
        McpTransport::Http {
            url: "http://127.0.0.1:9/mcp".into(),
        },
    );
    assert_eq!(
        server_refusal(&local),
        None,
        "a local address needs no consent"
    );
}

#[test]
fn usable_tools_go_into_the_instructions_and_a_call_is_read_back() {
    let s = server(
        "zotero",
        McpTransport::Stdio {
            command: "z".into(),
            args: vec![],
        },
    );
    let mut off = server(
        "off",
        McpTransport::Stdio {
            command: "o".into(),
            args: vec![],
        },
    );
    off.enabled = false;
    let found = vec![
        Discovered {
            server: "zotero".into(),
            tools: vec![tool("lookup", true)],
            ..Discovered::default()
        },
        Discovered {
            server: "off".into(),
            tools: vec![tool("hidden", true)],
            ..Discovered::default()
        },
    ];
    let block = tools_block(&[s.clone(), off], &found);
    assert!(block.contains("\"tool_call\"") && block.contains("lookup"));
    assert!(
        !block.contains("hidden"),
        "a switched-off server's tools are not offered"
    );
    assert_eq!(tools_block(&[s], &[]), "");
    let call = parse_tool_call("Sure.\n```json\n{\"tool_call\": {\"server\": \"zotero\", \"tool\": \"lookup\", \"arguments\": {\"q\": \"cells\"}}}\n```").unwrap();
    assert_eq!(
        (call.server.as_str(), call.tool.as_str()),
        ("zotero", "lookup")
    );
    assert_eq!(call.arguments, json!({"q": "cells"}));
    assert_eq!(
        parse_tool_call("{\"summary\": \"x\", \"changes\": []}"),
        None
    );
    assert_eq!(
        parse_tool_call("{\"tool_call\": {\"server\": \"z\", \"tool\": \"t\", \"arguments\": 3}}"),
        None
    );
    let done = result_message(&call, &Ok(("2 papers".into(), false)));
    assert!(done.contains("Tool lookup on zotero (result):\n2 papers"));
    let refused = result_message(&call, &Err("the user did not allow this call".into()));
    assert!(refused.contains("(not run)"));
}
