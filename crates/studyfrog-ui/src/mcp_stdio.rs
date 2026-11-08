//! The headless Model Context Protocol server (STORY-0120): `studyfrog --mcp [--vault PATH]
//! [--mcp-allow-write] [--mcp-allow-destructive]` reads one JSON-RPC message per line from standard
//! input and writes one answer per line to standard output. Only protocol messages are ever written to
//! standard output; everything else goes to standard error.

use std::io::{self, BufRead, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use studyfrog_agent::mcp_route::handle_line;
use studyfrog_agent::mcp_types::{McpPolicy, McpRegistry};
use studyfrog_service::session::open_vault;
use studyfrog_store::app_config::read_app_settings;

use crate::mcp_vault::{VaultCtx, vault_registry};
use crate::runtime::paths;

/// What the command line asked for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct McpArgs {
    /// The vault to open; else the one opened last.
    pub vault: Option<String>,
    /// What the server lets through.
    pub policy: McpPolicy,
    /// `--mcp-http 127.0.0.1:<port>`: serve over local HTTP instead of standard input and output.
    pub http: Option<String>,
}

/// Read the flags (the arguments after the program name); `--mcp` itself is skipped, anything unknown
/// is an error.
pub fn parse_mcp_args(args: &[String]) -> Result<McpArgs, String> {
    let mut out = McpArgs::default();
    let mut rest = args.iter().filter(|a| *a != "--mcp");
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--vault" => {
                let path = rest.next().ok_or("--vault needs a folder")?;
                out.vault = Some(path.clone());
            }
            "--mcp-http" => {
                let addr = rest.next().ok_or("--mcp-http needs 127.0.0.1:<port>")?;
                out.http = Some(addr.clone());
            }
            "--mcp-allow-write" => out.policy.allow_write = true,
            "--mcp-allow-destructive" => out.policy.allow_destructive = true,
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(out)
}

/// Answer every line of `input` on `output`, flushing after each answer, until the input ends.
/// Blank lines are skipped; a notification gets no answer.
pub fn serve_lines<C, R: BufRead, W: Write>(
    reg: &McpRegistry<C>,
    policy: &McpPolicy,
    ctx: &mut C,
    input: R,
    mut output: W,
) -> io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = handle_line(reg, policy, ctx, &line) {
            writeln!(output, "{reply}")?;
            output.flush()?;
        }
    }
    Ok(())
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

/// The vault to open: the one named, else the one opened last.
fn vault_path(args: &McpArgs) -> Result<String, String> {
    if let Some(path) = &args.vault {
        return Ok(path.clone());
    }
    let config = paths()
        .map(|p| p.config)
        .ok_or("no home directory is known")?;
    let settings = read_app_settings(&config).map_err(|e| e.to_string())?;
    settings
        .vault
        .ok_or_else(|| "no vault: give --vault PATH or open one in the app first".to_owned())
}

/// Read the flags (the program name first) and open the vault they name.
pub fn prepare(args: &[String]) -> Result<(McpArgs, VaultCtx), String> {
    let a = parse_mcp_args(args.get(1..).unwrap_or_default())?;
    let path = vault_path(&a)?;
    let (session, _) = open_vault(&path).map_err(|e| format!("{path}: {e}"))?;
    let policy = a.policy;
    Ok((
        a,
        VaultCtx {
            session,
            now_ms,
            policy,
        },
    ))
}

/// Run the server on standard input and output until the input ends; the exit code.
pub fn run(args: &[String]) -> u8 {
    let result = prepare(args).and_then(|(a, ctx)| match a.http {
        Some(_) => Err("--mcp-http is served by the studyfrog binary".to_owned()),
        None => Ok((a.policy, ctx)),
    });
    let (policy, mut ctx) = match result {
        Ok(ready) => ready,
        Err(e) => {
            eprintln!("studyfrog --mcp: {e}");
            return 1;
        }
    };
    let reg = vault_registry(env!("CARGO_PKG_VERSION"));
    let served = serve_lines(
        &reg,
        &policy,
        &mut ctx,
        io::stdin().lock(),
        io::stdout().lock(),
    );
    match served {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("studyfrog --mcp: {e}");
            1
        }
    }
}

/// The block an AI client (Claude Desktop, Cursor, Zed) takes in its settings to start this server
/// for `vault`, as JSON text.
pub fn mcp_config_snippet(command: &str, vault: &str) -> String {
    let config = serde_json::json!({
        "mcpServers": {"studyfrog": {"command": command, "args": ["--mcp", "--vault", vault]}}
    });
    serde_json::to_string_pretty(&config).unwrap_or_default()
}
