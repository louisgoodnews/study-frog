---
id: STORY-0120
type: story
title: CLI stdio MCP server mode and localhost Streamable HTTP transport
status: review
epic: EPIC-037
parents: [EPIC-037]
estimate: 5
labels: [m12, app, net, service]
acceptance_criteria:
  - "Running studyfrog-app --mcp [--vault <path>] [--mcp-allow-write] starts a headless JSON-RPC 2.0 MCP server over stdin/stdout with all diagnostic output routed strictly to stderr"
  - "An optional localhost-only Streamable HTTP transport (--mcp-http 127.0.0.1:<port>) enforces bearer-token authentication and Origin header validation to prevent DNS-rebinding or cross-origin requests"
  - "The Settings / Assistant screen displays a ready-to-copy MCP configuration snippet (claude_desktop_config.json / Cursor / Zed) for the active vault"
---

## Context
For external MCP hosts (Claude Desktop, Cursor, Zed, Windsurf, Cline, or CLI agents) to connect to StudyFrog, the application binary (`studyfrog-app`) needs a standard `stdio` mode (`studyfrog-app --mcp --vault /path/to/vault`) as well as an optional localhost Streamable HTTP listener when running alongside the desktop UI, plus a one-click configuration snippet generator in the UI so users can connect their favorite AI IDE in seconds.

## Description
Implement the `stdio` and localhost Streamable HTTP MCP transports and UI config helper:
1. **Headless `stdio` MCP Server (`crates/studyfrog-app`):**
   - Support CLI flags:
     - `--mcp`: Run the headless MCP server loop reading newline-delimited JSON-RPC 2.0 messages from `stdin` and writing responses to `stdout` (flushing after each line).
     - `--vault <path>`: Open the specified vault path (or default to the last opened vault from `app.toml`).
     - `--mcp-allow-write` / `--mcp-allow-destructive`: Configure `McpPolicy`.
   - Ensure zero non-protocol bytes ever touch `stdout` (any warnings/errors go exclusively to `stderr`).
2. **Localhost Streamable HTTP Transport (`crates/studyfrog-net`):**
   - Optional local listener bound strictly to `127.0.0.1` (never `0.0.0.0`), verifying an ephemeral or `secrets.toml` bearer token and rejecting browser `Origin` headers outside localhost.
3. **One-Click MCP Client Config Snippet (`crates/studyfrog-ui`):**
   - In the Assistant `Connections` tab, show a `"Copy MCP server config"` button that copies the ready-to-paste JSON block:
     ```json
     {
       "mcpServers": {
         "studyfrog": {
           "command": "studyfrog-app",
           "args": ["--mcp", "--vault", "/path/to/vault"]
         }
       }
     }
     ```

## Acceptance Criteria
- [x] Running studyfrog-app --mcp [--vault <path>] [--mcp-allow-write] starts a headless JSON-RPC 2.0 MCP server over stdin/stdout with all diagnostic output routed strictly to stderr
- [ ] An optional localhost-only Streamable HTTP transport (--mcp-http 127.0.0.1:<port>) enforces bearer-token authentication and Origin header validation to prevent DNS-rebinding or cross-origin requests
- [x] The Settings / Assistant screen displays a ready-to-copy MCP configuration snippet (claude_desktop_config.json / Cursor / Zed) for the active vault

## Design Notes
- Touchpoints:
  - `crates/studyfrog-app/src/mcp_stdio.rs`, `crates/studyfrog-net/src/`, `crates/studyfrog-ui/src/view/agent.rs`.
- Comply with P1 (zero inherent `impl` blocks), P2 (≤ 250 lines per file), and P3 (`check-deps.sh`).

## Progress (2026-11-08)
Built and tried: `studyfrog --mcp [--vault PATH] [--mcp-allow-write] [--mcp-allow-destructive]` serves the vault over standard input and output (newline-delimited JSON-RPC, one flushed line per answer, nothing but protocol on standard output, problems on standard error with exit code 1); the vault defaults to the one opened last. It was run as the real binary over pipes. The Assistant's connections tab has "Copy MCP server config" with the block for the open vault. The binary is called `studyfrog`, not `studyfrog-app` as the story says.

Open: the localhost Streamable HTTP transport with its token and origin checks (criterion 2) is not built.

## Dependencies
EPIC-037, STORY-0117, STORY-0118, STORY-0119.

## Open Questions
None.
