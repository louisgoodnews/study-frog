//! Talking to an external MCP server for the assistant (STORY-0121): a program started with its
//! standard input and output as the channel, or a Streamable HTTP endpoint reached through the
//! application's transport. Each discovery or tool call opens its own short session.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use studyfrog_agent::mcp_client::{
    Discovered, McpServer, McpTransport, call_text, client_notification, client_request,
    http_messages, info_of, initialize_params, names_of, response_for, server_refusal, tools_of,
};
use studyfrog_agent::mcp_types::PROTOCOL_VERSION;
use studyfrog_agent::provider::{HttpRequest, HttpResponse, Transport};

/// How long one answer may take.
pub const ANSWER_TIMEOUT: Duration = Duration::from_secs(30);

/// A running server program.
pub struct StdioLink {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<String>,
    next_id: u64,
    timeout: Duration,
}

/// Start the program; its standard error is discarded.
pub fn stdio_open(command: &str, args: &[String], timeout: Duration) -> Result<StdioLink, String> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot start {command}: {e}"))?;
    let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        let _ = child.kill();
        return Err(format!("cannot talk to {command}"));
    };
    let (tx, lines) = channel();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    Ok(StdioLink {
        child,
        stdin,
        lines,
        next_id: 1,
        timeout,
    })
}

fn send_line(link: &mut StdioLink, line: &str) -> Result<(), String> {
    writeln!(link.stdin, "{line}")
        .and_then(|()| link.stdin.flush())
        .map_err(|e| format!("the server stopped listening: {e}"))
}

/// The answer to request `id`; lines that are not that answer (logs, notifications) are skipped.
fn wait_for(link: &StdioLink, id: u64) -> Result<Value, String> {
    let end = Instant::now() + link.timeout;
    loop {
        let left = end.saturating_duration_since(Instant::now());
        let line = match link.lines.recv_timeout(left) {
            Ok(line) => line,
            Err(RecvTimeoutError::Timeout) => {
                return Err("the server did not answer in time".to_owned());
            }
            Err(RecvTimeoutError::Disconnected) => return Err("the server ended".to_owned()),
        };
        let found = serde_json::from_str::<Value>(&line)
            .ok()
            .and_then(|m| response_for(&m, id));
        if let Some(answer) = found {
            return answer;
        }
    }
}

/// Send a request and wait for its answer.
pub fn stdio_request(link: &mut StdioLink, method: &str, params: &Value) -> Result<Value, String> {
    let id = link.next_id;
    link.next_id += 1;
    send_line(link, &client_request(id, method, params))?;
    wait_for(link, id)
}

/// Stop the program.
pub fn stdio_close(mut link: StdioLink) {
    let _ = link.child.kill();
    let _ = link.child.wait();
}

fn http_post(
    transport: Transport,
    url: &str,
    session: Option<&str>,
    body: String,
) -> Result<HttpResponse, String> {
    let mut headers = vec![
        ("Content-Type".to_owned(), "application/json".to_owned()),
        (
            "Accept".to_owned(),
            "application/json, text/event-stream".to_owned(),
        ),
        (
            "MCP-Protocol-Version".to_owned(),
            PROTOCOL_VERSION.to_owned(),
        ),
    ];
    headers.extend(session.map(|s| ("Mcp-Session-Id".to_owned(), s.to_owned())));
    let request = HttpRequest {
        url: url.to_owned(),
        headers,
        body,
        timeout_secs: 30,
    };
    transport(&request)
}

/// Send one request over HTTP (with the session the server gave, if any) and read its answer, plain
/// JSON or server-sent events; also the session id the answer names.
pub fn http_request(
    transport: Transport,
    (url, session): (&str, Option<&str>),
    (id, method, params): (u64, &str, &Value),
) -> Result<(Value, Option<String>), String> {
    let response = http_post(transport, url, session, client_request(id, method, params))?;
    if !(200..300).contains(&response.status) {
        return Err(format!(
            "the server answered with status {}",
            response.status
        ));
    }
    let named = response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("mcp-session-id"))
        .map(|(_, v)| v.clone());
    let answer = http_messages(&response.body)
        .iter()
        .find_map(|m| response_for(m, id))
        .unwrap_or_else(|| Err("the server's answer holds no result".to_owned()))?;
    Ok((answer, named))
}

/// Ask a server: `method` with `params`, the result or why not.
pub type Ask<'a> = dyn FnMut(&str, &Value) -> Result<Value, String> + 'a;

/// Discover what a server offers through `ask` (after `initialize`). Lists a server does not
/// support are left empty.
pub fn discover_with(name: &str, ask: &mut Ask<'_>, init: &Value) -> Discovered {
    let mut list = |method: &str| ask(method, &json!({})).unwrap_or(Value::Null);
    let tools = tools_of(&list("tools/list"));
    let mut resources = names_of(&list("resources/list"), "resources", "uri");
    resources.extend(names_of(
        &list("resources/templates/list"),
        "resourceTemplates",
        "uriTemplate",
    ));
    let prompts = names_of(&list("prompts/list"), "prompts", "name");
    Discovered {
        server: name.to_owned(),
        info: info_of(init),
        tools,
        resources,
        prompts,
    }
}

/// Open a session with the server, run `work` in it, and close it.
fn with_session<T>(
    server: &McpServer,
    transport: Option<Transport>,
    work: impl FnOnce(&mut Ask<'_>, &Value) -> T,
) -> Result<T, String> {
    if let Some(why) = server_refusal(server) {
        return Err(why);
    }
    let init = initialize_params(env!("CARGO_PKG_VERSION"));
    match &server.transport {
        McpTransport::Stdio { command, args } => {
            let mut link = stdio_open(command, args, ANSWER_TIMEOUT)?;
            let started = stdio_request(&mut link, "initialize", &init);
            let out = started.and_then(|answer| {
                send_line(&mut link, &client_notification("notifications/initialized"))?;
                let mut ask = |m: &str, p: &Value| stdio_request(&mut link, m, p);
                Ok(work(&mut ask, &answer))
            });
            stdio_close(link);
            out
        }
        McpTransport::Http { url } => {
            let transport = transport.ok_or("This build cannot reach the network.")?;
            let (answer, session) = http_request(transport, (url, None), (1, "initialize", &init))?;
            let session = session.as_deref();
            let note = client_notification("notifications/initialized");
            let _ = http_post(transport, url, session, note);
            let mut id = 1;
            let mut ask = |m: &str, p: &Value| {
                id += 1;
                http_request(transport, (url, session), (id, m, p)).map(|(v, _)| v)
            };
            Ok(work(&mut ask, &answer))
        }
    }
}

/// What a server offers.
pub fn discover_server(
    server: &McpServer,
    transport: Option<Transport>,
) -> Result<Discovered, String> {
    with_session(server, transport, |ask, init| {
        discover_with(&server.name, ask, init)
    })
}

/// Call a tool of a server: its text and whether it is an error.
pub fn call_server_tool(
    server: &McpServer,
    transport: Option<Transport>,
    tool: &str,
    arguments: &Value,
) -> Result<(String, bool), String> {
    let params = json!({"name": tool, "arguments": arguments});
    with_session(server, transport, |ask, _| {
        ask("tools/call", &params).map(|r| call_text(&r))
    })?
}
