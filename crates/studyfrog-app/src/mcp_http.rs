//! `studyfrog --mcp --mcp-http 127.0.0.1:<port> [--vault PATH] [--mcp-allow-write]`: the MCP server
//! on a local Streamable HTTP listener. The bearer token is `mcp_http` under `[keys]` in
//! `secrets.toml`, else a fresh one printed to standard error; nothing else is printed.

use std::net::TcpListener;
use std::path::Path;

use studyfrog_agent::settings::{SECRETS_FILE, parse_secrets};
use studyfrog_net::mcp_http::{HttpGuard, MCP_PATH, ephemeral_token, loopback_addr, serve};
use studyfrog_ui::mcp_stdio::prepare;
use studyfrog_ui::mcp_vault::vault_registry;

/// The name of the token in `secrets.toml`.
pub const TOKEN_KEY: &str = "mcp_http";

/// The token from `secrets.toml` in `config`, if one is set there.
pub fn saved_token(config: &Path) -> Option<String> {
    let text = std::fs::read_to_string(config.join(SECRETS_FILE)).ok()?;
    parse_secrets(&text)
        .ok()?
        .remove(TOKEN_KEY)
        .filter(|t| !t.trim().is_empty())
}

fn start(args: &[String]) -> Result<u8, String> {
    let (a, mut ctx) = prepare(args)?;
    let text = a
        .http
        .as_deref()
        .ok_or("--mcp-http needs 127.0.0.1:<port>")?;
    let addr = loopback_addr(text)?;
    let listener = TcpListener::bind(addr).map_err(|e| format!("{addr}: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let config = studyfrog_ui::runtime::paths().map(|p| p.config);
    let token = match config.as_deref().and_then(saved_token) {
        Some(t) => {
            eprintln!("studyfrog --mcp-http: using the token `{TOKEN_KEY}` from {SECRETS_FILE}");
            t
        }
        None => {
            let t = ephemeral_token().map_err(|e| format!("no random token: {e}"))?;
            eprintln!("studyfrog --mcp-http: bearer token for this run: {t}");
            t
        }
    };
    eprintln!("studyfrog --mcp-http: serving http://127.0.0.1:{port}{MCP_PATH}");
    let guard = HttpGuard { token, port };
    let reg = vault_registry(env!("CARGO_PKG_VERSION"));
    serve(&listener, &guard, &reg, &a.policy, &mut ctx, None);
    Ok(0)
}

/// Run the listener until the process is stopped; the exit code.
pub fn run(args: &[String]) -> u8 {
    start(args).unwrap_or_else(|e| {
        eprintln!("studyfrog --mcp-http: {e}");
        1
    })
}
