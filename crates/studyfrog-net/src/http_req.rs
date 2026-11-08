//! A small HTTP/1.1 request reader and response writer for the local MCP listener: one request per
//! connection, a body only by `Content-Length`, and limits on every part. Written in-house because
//! the listener needs a few dozen lines of HTTP, not a server framework (P10).

use std::io::{self, BufRead, Read, Write};

/// Longest request line or header line accepted.
pub const MAX_LINE: usize = 8 * 1024;
/// Most headers accepted.
pub const MAX_HEADERS: usize = 64;
/// Largest body accepted.
pub const MAX_BODY: usize = 4 * 1024 * 1024;

/// A request as read.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HttpReq {
    /// `GET`, `POST`, ...
    pub method: String,
    /// The path, without the query.
    pub path: String,
    /// Header names in lower case, with their values.
    pub headers: Vec<(String, String)>,
    /// The body.
    pub body: Vec<u8>,
}

/// Why a request could not be read, as the status to answer with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The HTTP status.
    pub status: u16,
    /// A sentence for the client.
    pub reason: String,
}

fn refuse(status: u16, reason: &str) -> Refusal {
    Refusal {
        status,
        reason: reason.to_owned(),
    }
}

/// The value of header `name` (lower case), if given.
pub fn header<'a>(req: &'a HttpReq, name: &str) -> Option<&'a str> {
    req.headers
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.as_str())
}

fn line(input: &mut impl BufRead) -> Result<String, Refusal> {
    let mut buf = Vec::new();
    let read = input
        .take(MAX_LINE as u64 + 1)
        .read_until(b'\n', &mut buf)
        .map_err(|_| refuse(400, "the request could not be read"))?;
    if read > MAX_LINE {
        return Err(refuse(431, "a header line is too long"));
    }
    let text = String::from_utf8(buf).map_err(|_| refuse(400, "the request is not text"))?;
    Ok(text.trim_end_matches(['\r', '\n']).to_owned())
}

fn headers(input: &mut impl BufRead) -> Result<Vec<(String, String)>, Refusal> {
    let mut out = Vec::new();
    loop {
        let l = line(input)?;
        if l.is_empty() {
            return Ok(out);
        }
        if out.len() == MAX_HEADERS {
            return Err(refuse(431, "too many headers"));
        }
        let (name, value) = l
            .split_once(':')
            .ok_or_else(|| refuse(400, "a header has no colon"))?;
        out.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
    }
}

/// Read one request from `input`.
pub fn read_request(input: &mut impl BufRead) -> Result<HttpReq, Refusal> {
    let first = line(input)?;
    let mut parts = first.split(' ');
    let (method, target, version) = (parts.next(), parts.next(), parts.next());
    let (Some(method), Some(target), Some(v)) = (method, target, version) else {
        return Err(refuse(400, "not an HTTP request line"));
    };
    if !v.starts_with("HTTP/1.") {
        return Err(refuse(505, "only HTTP/1.x is spoken here"));
    }
    let mut req = HttpReq {
        method: method.to_owned(),
        path: target.split('?').next().unwrap_or("").to_owned(),
        headers: headers(input)?,
        body: Vec::new(),
    };
    if header(&req, "transfer-encoding").is_some() {
        return Err(refuse(501, "send the body with Content-Length"));
    }
    let len: usize = match header(&req, "content-length") {
        None => 0,
        Some(n) => n.parse().map_err(|_| refuse(400, "a bad Content-Length"))?,
    };
    if len > MAX_BODY {
        return Err(refuse(413, "the body is too large"));
    }
    req.body.resize(len, 0);
    input
        .read_exact(&mut req.body)
        .map_err(|_| refuse(400, "the body ended early"))?;
    Ok(req)
}

/// The reason phrase of the statuses this listener sends.
pub fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        431 => "Request Header Fields Too Large",
        501 => "Not Implemented",
        505 => "HTTP Version Not Supported",
        _ => "Error",
    }
}

/// Write a whole response and close the exchange; `extra` are more header lines (`Name: value`).
pub fn write_response(
    out: &mut impl Write,
    status: u16,
    content_type: &str,
    extra: &[String],
    body: &str,
) -> io::Result<()> {
    write!(out, "HTTP/1.1 {status} {}\r\n", status_text(status))?;
    if !body.is_empty() {
        write!(out, "Content-Type: {content_type}\r\n")?;
    }
    write!(
        out,
        "Content-Length: {}\r\nConnection: close\r\n",
        body.len()
    )?;
    write!(
        out,
        "Cache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n"
    )?;
    for h in extra {
        write!(out, "{h}\r\n")?;
    }
    write!(out, "\r\n{body}")?;
    out.flush()
}
