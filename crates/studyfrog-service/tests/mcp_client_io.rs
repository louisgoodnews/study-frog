#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The assistant's MCP client against fake local servers: a shell script over standard input and
//! output, StudyFrog's own router in-process, and an HTTP endpoint answering with server-sent events.

use std::time::Duration;

use serde_json::{Value, json};
use studyfrog_agent::mcp_client::{McpServer, McpTransport};
use studyfrog_agent::mcp_route::handle_message;
use studyfrog_agent::mcp_types::{
    McpContent, McpError, McpPolicy, McpTool, McpToolAnnotations, register_tool, registry_new,
};
use studyfrog_agent::provider::{HttpRequest, HttpResponse};
use studyfrog_service::mcp_client_io::{
    call_server_tool, discover_server, discover_with, stdio_close, stdio_open, stdio_request,
};

fn script() -> String {
    format!(
        "{}/tests/fixtures/fake_mcp_server.sh",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn fake(enabled: bool) -> McpServer {
    McpServer {
        name: "fake".into(),
        transport: McpTransport::Stdio {
            command: "sh".into(),
            args: vec![script()],
        },
        enabled,
        trust_read_only: false,
        allowed: false,
    }
}

#[test]
fn a_program_is_discovered_and_its_tools_are_called_over_its_standard_streams() {
    let found = discover_server(&fake(true), None).unwrap();
    assert_eq!(found.info, "fake 0.1");
    let tools: Vec<(&str, bool)> = found
        .tools
        .iter()
        .map(|t| (t.name.as_str(), t.read_only))
        .collect();
    assert_eq!(
        tools,
        [("lookup", true), ("save", false)],
        "no annotation counts as changing"
    );
    assert_eq!(
        found.resources,
        ["fake://library"],
        "the templates list failed and is empty"
    );
    assert_eq!(found.prompts, ["cite"]);
    let ran = call_server_tool(&fake(true), None, "save", &json!({"q": "x"})).unwrap();
    assert_eq!(ran, ("ran save".to_owned(), false));
    let off = discover_server(&fake(false), None).unwrap_err();
    assert!(off.contains("switched off"), "{off}");
}

#[test]
fn a_session_skips_log_lines_reports_errors_and_times_out() {
    let mut link = stdio_open("sh", &[script()], Duration::from_secs(10)).unwrap();
    let err = stdio_request(&mut link, "nothing/here", &json!({})).unwrap_err();
    assert!(err.contains("no such method"), "{err}");
    let ok = stdio_request(&mut link, "prompts/list", &json!({})).unwrap();
    assert_eq!(
        ok["prompts"][0]["name"], "cite",
        "the log line before the error was skipped"
    );
    stdio_close(link);
    let mut silent = stdio_open(
        "sh",
        &["-c".into(), "sleep 5".into()],
        Duration::from_millis(200),
    )
    .unwrap();
    let late = stdio_request(&mut silent, "ping", &json!({})).unwrap_err();
    assert!(late.contains("in time"), "{late}");
    stdio_close(silent);
    assert!(stdio_open("/no/such/program", &[], Duration::from_secs(1)).is_err());
}

fn shout(_: &mut (), args: &Value) -> Result<McpContent, McpError> {
    let text = args
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_uppercase();
    Ok(McpContent {
        text,
        is_error: false,
    })
}

#[test]
fn studyfrogs_own_router_serves_as_an_in_process_server() {
    let tool = McpTool {
        name: "shout",
        description: "Upper case",
        input_schema: r#"{"type":"object"}"#,
        annotations: McpToolAnnotations {
            title: "Shout",
            read_only_hint: true,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        },
        staged: false,
        call: shout,
    };
    let reg = register_tool(registry_new("inner", "2.0"), tool);
    let mut id = 0;
    let mut ask = |method: &str, params: &Value| {
        id += 1;
        let message = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        let answer = handle_message(&reg, &McpPolicy::default(), &mut (), &message).unwrap();
        studyfrog_agent::mcp_client::response_for(&answer, id).unwrap()
    };
    let init = ask("initialize", &json!({})).unwrap();
    let found = discover_with("inner", &mut ask, &init);
    assert_eq!(found.info, "inner 2.0");
    assert!(found.tools[0].read_only);
    let r = ask(
        "tools/call",
        &json!({"name": "shout", "arguments": {"text": "hi"}}),
    )
    .unwrap();
    assert_eq!(
        studyfrog_agent::mcp_client::call_text(&r),
        ("HI".to_owned(), false)
    );
}

fn answer(
    status: u16,
    body: String,
    headers: Vec<(String, String)>,
) -> Result<HttpResponse, String> {
    Ok(HttpResponse {
        status,
        body,
        headers,
    })
}

/// A Streamable HTTP server that hands out a session on `initialize` and wants it back on every
/// later message, and answers with server-sent events.
fn sse_server(req: &HttpRequest) -> Result<HttpResponse, String> {
    let v: Value = serde_json::from_str(&req.body).unwrap();
    let header = |name: &str| {
        req.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    };
    assert!(header("Accept").unwrap().contains("text/event-stream"));
    let method = v["method"].as_str().unwrap();
    if method != "initialize" && header("Mcp-Session-Id") != Some("s-1") {
        return answer(400, "no session".into(), Vec::new());
    }
    let result = match method {
        "initialize" => json!({"serverInfo": {"name": "web", "version": "1"}}),
        "notifications/initialized" => return answer(202, String::new(), Vec::new()),
        "tools/list" => {
            json!({"tools": [{"name": "search", "annotations": {"readOnlyHint": true}}]})
        }
        "tools/call" => json!({"content": [{"type": "text", "text": "3 papers"}]}),
        _ => {
            let e =
                json!({"jsonrpc": "2.0", "id": v["id"], "error": {"code": -1, "message": "no"}});
            return answer(200, e.to_string(), Vec::new());
        }
    };
    let event = json!({"jsonrpc": "2.0", "id": v["id"], "result": result});
    let session = vec![("mcp-session-id".to_owned(), "s-1".to_owned())];
    answer(200, format!("event: message\ndata: {event}\n\n"), session)
}

#[test]
fn an_http_server_needs_consent_when_remote_and_reads_server_sent_events() {
    let mut s = McpServer {
        name: "web".into(),
        transport: McpTransport::Http {
            url: "https://tools.example.org/mcp".into(),
        },
        enabled: true,
        trust_read_only: true,
        allowed: false,
    };
    let refused = discover_server(&s, Some(sse_server)).unwrap_err();
    assert!(refused.contains("until you allow it"), "{refused}");
    s.allowed = true;
    let found = discover_server(&s, Some(sse_server)).unwrap();
    assert_eq!((found.info.as_str(), found.tools.len()), ("web 1", 1));
    let ran = call_server_tool(&s, Some(sse_server), "search", &json!({})).unwrap();
    assert_eq!(ran.0, "3 papers");
    assert!(
        discover_server(&s, None)
            .unwrap_err()
            .contains("cannot reach the network")
    );
}
