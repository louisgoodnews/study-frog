//! `mcp_servers.toml` in the config folder of the application home (STORY-0121): the external MCP
//! servers the assistant may use. No keys are kept here.

use serde::Deserialize;

use crate::mcp_client::{McpServer, McpTransport};

/// File name of the servers.
pub const MCP_SERVERS_FILE: &str = "mcp_servers.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawServers {
    #[serde(default)]
    server: Vec<RawServer>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawServer {
    name: String,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default = "yes")]
    enabled: bool,
    #[serde(default)]
    trust_read_only: bool,
    #[serde(default)]
    allowed: bool,
}

fn yes() -> bool {
    true
}

fn one_line(s: &str) -> bool {
    !s.trim().is_empty() && !s.chars().any(char::is_control)
}

/// Why a server cannot be used, in words; `None` when it is fine.
pub fn server_problem(s: &McpServer, others: &[McpServer]) -> Option<String> {
    let name_ok = one_line(&s.name)
        && s.name.chars().count() <= 40
        && s.name
            .chars()
            .all(|c| c.is_alphanumeric() || "-_ ".contains(c));
    if !name_ok {
        return Some(
            "The name is one line of letters, digits, spaces, - and _ (at most 40).".to_owned(),
        );
    }
    if others.iter().any(|o| o.name.eq_ignore_ascii_case(&s.name)) {
        return Some(format!(
            "There is already a tool server called \"{}\".",
            s.name
        ));
    }
    match &s.transport {
        McpTransport::Stdio { command, args }
            if !one_line(command) || args.iter().any(|a| a.chars().any(char::is_control)) =>
        {
            Some("Give the program to start, on one line.".to_owned())
        }
        McpTransport::Http { url }
            if !(url.starts_with("http://") || url.starts_with("https://")) || !one_line(url) =>
        {
            Some("The address must start with http:// or https://.".to_owned())
        }
        _ => None,
    }
}

fn server_of(r: RawServer) -> Result<McpServer, String> {
    let transport = match (r.command, r.url) {
        (Some(command), None) => McpTransport::Stdio {
            command,
            args: r.args,
        },
        (None, Some(url)) => McpTransport::Http { url },
        _ => {
            return Err(format!(
                "the server \"{}\" needs either a command or a url",
                r.name
            ));
        }
    };
    Ok(McpServer {
        name: r.name,
        transport,
        enabled: r.enabled,
        trust_read_only: r.trust_read_only,
        allowed: r.allowed,
    })
}

/// Parse `mcp_servers.toml`.
pub fn parse_servers(text: &str) -> Result<Vec<McpServer>, String> {
    let raw: RawServers = toml::from_str(text).map_err(|e| e.message().to_owned())?;
    let mut out: Vec<McpServer> = Vec::new();
    for r in raw.server {
        let s = server_of(r)?;
        if let Some(why) = server_problem(&s, &out) {
            return Err(why);
        }
        out.push(s);
    }
    Ok(out)
}

fn quoted(s: &str) -> String {
    toml::Value::String(s.to_owned()).to_string()
}

/// The text of `mcp_servers.toml`.
pub fn render_servers(list: &[McpServer]) -> String {
    let mut out = String::from("# External MCP tool servers the StudyFrog assistant may use.\n");
    for s in list {
        out.push_str(&format!("\n[[server]]\nname = {}\n", quoted(&s.name)));
        match &s.transport {
            McpTransport::Stdio { command, args } => {
                let args: Vec<String> = args.iter().map(|a| quoted(a)).collect();
                out.push_str(&format!(
                    "command = {}\nargs = [{}]\n",
                    quoted(command),
                    args.join(", ")
                ));
            }
            McpTransport::Http { url } => out.push_str(&format!("url = {}\n", quoted(url))),
        }
        out.push_str(&format!(
            "enabled = {}\ntrust_read_only = {}\nallowed = {}\n",
            s.enabled, s.trust_read_only, s.allowed
        ));
    }
    out
}

/// A command line split into the program and its arguments; double quotes keep spaces together.
pub fn split_command(line: &str) -> Option<(String, Vec<String>)> {
    let mut words: Vec<String> = Vec::new();
    let (mut word, mut quoted, mut started) = (String::new(), false, false);
    for c in line.chars() {
        match c {
            '"' => (quoted, started) = (!quoted, true),
            c if c.is_whitespace() && !quoted => {
                if started {
                    words.push(std::mem::take(&mut word));
                }
                started = false;
            }
            c => (word, started) = (word + &c.to_string(), true),
        }
    }
    if started {
        words.push(word);
    }
    let mut it = words.into_iter();
    it.next().map(|program| (program, it.collect()))
}
