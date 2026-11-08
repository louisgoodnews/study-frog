//! The localhost Streamable HTTP transport of the MCP server (STORY-0120): `POST /mcp` with one
//! JSON-RPC message answers with one JSON message (`202 Accepted` for a notification). It listens on
//! 127.0.0.1 only, wants `Authorization: Bearer <token>`, and refuses a `Host` or browser `Origin`
//! that is not this machine, so a web page cannot reach it through DNS rebinding.

use std::io::{BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::time::Duration;

use studyfrog_agent::mcp_route::handle_line;
use studyfrog_agent::mcp_types::{McpPolicy, McpRegistry};

use crate::http_req::{HttpReq, Refusal, header, read_request, write_response};

/// The only path served.
pub const MCP_PATH: &str = "/mcp";
/// How long a client may take to send its request.
pub const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// What a request must match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpGuard {
    /// The bearer token.
    pub token: String,
    /// The port listened on (the `Host` header must name it).
    pub port: u16,
}

/// The address to listen on: `127.0.0.1:<port>` and nothing else (never all interfaces).
pub fn loopback_addr(text: &str) -> Result<SocketAddr, String> {
    let addr: SocketAddr = text
        .parse()
        .map_err(|_| format!("{text}: give 127.0.0.1:<port>"))?;
    match addr {
        SocketAddr::V4(v4) if *v4.ip() == Ipv4Addr::LOCALHOST => Ok(addr),
        _ => Err(format!("{text}: the MCP listener binds to 127.0.0.1 only")),
    }
}

/// `host[:port]` split, with `[::1]` kept in brackets.
fn host_port(text: &str) -> (&str, Option<&str>) {
    if let Some(rest) = text.strip_prefix('[') {
        let end = rest.find(']').map_or(rest.len(), |i| i + 1);
        let (h, p) = text.split_at(end + 1);
        return (h, p.strip_prefix(':'));
    }
    text.split_once(':')
        .map_or((text, None), |(h, p)| (h, Some(p)))
}

fn is_local_name(host: &str) -> bool {
    matches!(
        host.to_ascii_lowercase().as_str(),
        "127.0.0.1" | "localhost" | "[::1]"
    )
}

/// Whether a `Host` header names this listener.
pub fn host_allowed(host: &str, port: u16) -> bool {
    let (h, p) = host_port(host.trim());
    is_local_name(h) && p == Some(port.to_string().as_str())
}

/// Whether a browser `Origin` is a page on this machine (`http://localhost:3000`, say).
pub fn origin_allowed(origin: &str) -> bool {
    let rest = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"));
    rest.is_some_and(|r| !r.contains('/') && is_local_name(host_port(r).0))
}

/// Compare tokens in time that does not depend on where they differ.
pub fn token_matches(given: &str, expected: &str) -> bool {
    let (a, b) = (given.as_bytes(), expected.as_bytes());
    let diff = a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y));
    a.len() == b.len() && diff == 0 && !b.is_empty()
}

fn refuse(status: u16, reason: &str) -> Refusal {
    Refusal {
        status,
        reason: reason.to_owned(),
    }
}

/// Whether the request may reach the server; else the status to refuse it with.
pub fn check(req: &HttpReq, guard: &HttpGuard) -> Result<(), Refusal> {
    if req.path != MCP_PATH {
        return Err(refuse(404, "the MCP endpoint is /mcp"));
    }
    if !header(req, "host").is_some_and(|h| host_allowed(h, guard.port)) {
        return Err(refuse(
            403,
            "the Host header must name 127.0.0.1 or localhost and this port",
        ));
    }
    if header(req, "origin").is_some_and(|o| !origin_allowed(o)) {
        return Err(refuse(403, "requests from other origins are refused"));
    }
    let bearer = header(req, "authorization").and_then(|a| {
        a.strip_prefix("Bearer ")
            .or_else(|| a.strip_prefix("bearer "))
    });
    if !bearer.is_some_and(|t| token_matches(t.trim(), &guard.token)) {
        return Err(refuse(401, "a valid bearer token is required"));
    }
    if req.method != "POST" {
        return Err(refuse(
            405,
            "only POST is served (no server-sent event stream)",
        ));
    }
    let json = header(req, "content-type").is_some_and(|c| c.starts_with("application/json"));
    if !json {
        return Err(refuse(415, "send the message as application/json"));
    }
    Ok(())
}

/// The status, extra headers and body for one request.
pub fn respond<C>(
    req: &HttpReq,
    guard: &HttpGuard,
    reg: &McpRegistry<C>,
    policy: &McpPolicy,
    ctx: &mut C,
) -> (u16, Vec<String>, String) {
    if let Err(r) = check(req, guard) {
        let extra = match r.status {
            401 => vec!["WWW-Authenticate: Bearer".to_owned()],
            405 => vec!["Allow: POST".to_owned()],
            _ => Vec::new(),
        };
        let body = format!("{{\"error\":{:?}}}", r.reason);
        return (r.status, extra, body);
    }
    let text = String::from_utf8_lossy(&req.body);
    match handle_line(reg, policy, ctx, &text) {
        Some(reply) => (200, Vec::new(), reply),
        None => (202, Vec::new(), String::new()),
    }
}

/// Serve one exchange on `stream`.
pub fn serve_one<C, S: Read + Write>(
    stream: &mut S,
    guard: &HttpGuard,
    reg: &McpRegistry<C>,
    policy: &McpPolicy,
    ctx: &mut C,
) -> std::io::Result<()> {
    let (status, extra, body) = match read_request(&mut BufReader::new(&mut *stream)) {
        Ok(req) => respond(&req, guard, reg, policy, ctx),
        Err(r) => (
            r.status,
            Vec::new(),
            format!("{{\"error\":{:?}}}", r.reason),
        ),
    };
    write_response(stream, status, "application/json", &extra, &body)
}

/// Accept connections one after another (the vault is used by one request at a time), until
/// `limit` connections were served (`None`: forever).
pub fn serve<C>(
    listener: &TcpListener,
    guard: &HttpGuard,
    reg: &McpRegistry<C>,
    policy: &McpPolicy,
    ctx: &mut C,
    limit: Option<usize>,
) {
    let mut left = limit;
    for stream in listener.incoming() {
        if let Ok(mut s) = stream {
            let _ = s.set_read_timeout(Some(READ_TIMEOUT));
            let _ = serve_one(&mut s, guard, reg, policy, ctx);
        }
        left = left.map(|n| n.saturating_sub(1));
        if left == Some(0) {
            return;
        }
    }
}

/// A fresh token: 32 random bytes from the system, as hex.
pub fn ephemeral_token() -> std::io::Result<String> {
    let mut bytes = [0u8; 32];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
