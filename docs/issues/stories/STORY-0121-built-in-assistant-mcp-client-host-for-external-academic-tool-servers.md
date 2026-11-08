---
id: STORY-0121
type: story
title: Built-in Assistant MCP client host for external academic tool servers
status: done
epic: EPIC-037
parents: [EPIC-037]
estimate: 8
labels: [m12, agent, net, service, ui]
acceptance_criteria:
  - "Users can register external MCP servers (local stdio command or remote Streamable HTTP URL) in the Assistant Connections tab and inspect their discovered tools, resources, and prompts"
  - "When running an Assistant task or interactive exam, StudyFrog can attach discovered MCP tool schemas to the LLM request and execute approved tool calls"
  - "External MCP tools without readOnlyHint: true require explicit user confirmation in the Assistant UI before execution"
---

## Context
Beyond acting as an MCP Server for external AI apps, StudyFrog's own built-in Assistant (`studyfrog-agent`) benefits from acting as an **MCP Host/Client**. Learners and researchers use specialized MCP servers—such as Zotero citation servers, arXiv/PubMed literature search, Wolfram/Python computation servers, or custom institutional databases—and want to connect them to StudyFrog's Assistant to verify facts, import citations, or enrich study notes.

## Description
Implement MCP Client/Host support inside StudyFrog's Assistant:
1. **External MCP Server Configuration (`crates/studyfrog-agent`, `crates/studyfrog-service`):**
   - Allow configuring external MCP servers in `.studyfrog/agent.toml` (or user config):
     - `name`: Unique label (e.g., `"zotero"`, `"arxiv"`).
     - `transport`: `Stdio { command, args }` or `Http { url }`.
     - `enabled`: `bool` and `trust_read_only: bool`.
2. **Capability Discovery & Tool Schema Bridging:**
   - Send `initialize`, `tools/list`, and `resources/list` to connected MCP servers.
   - Display discovered tools and their `ToolAnnotations` (`readOnlyHint`, `destructiveHint`) in the Assistant `Connections` tab.
3. **Human-in-the-Loop Tool Execution in Assistant Runs:**
   - When the connected LLM requests an external MCP tool call during an Assistant conversation:
     - If `readOnlyHint == true` and `trust_read_only == true`, execute via the MCP client transport and return the result to the conversation.
     - Otherwise, pause and display an inline approval card (`"Allow Assistant to run tool <name> on server <server>?"` with full JSON arguments preview) before executing.

## Acceptance Criteria
- [x] Users can register external MCP servers (local stdio command or remote Streamable HTTP URL) in the Assistant Connections tab and inspect their discovered tools, resources, and prompts
- [x] When running an Assistant task or interactive exam, StudyFrog can attach discovered MCP tool schemas to the LLM request and execute approved tool calls
- [x] External MCP tools without readOnlyHint: true require explicit user confirmation in the Assistant UI before execution

## Design Notes
- Touchpoints:
  - `crates/studyfrog-agent/src/mcp_client.rs`, `crates/studyfrog-service/src/agent.rs`, `crates/studyfrog-ui/src/view/agent.rs`.
- Strictly enforce P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Done. Built in `crates/studyfrog-agent/src/mcp_client.rs` (servers, consent, approval rule, client messages), `mcp_client_cfg.rs` (`mcp_servers.toml`), `mcp_client_chat.rs` (tools in the instructions, the model's tool call, the result turn); `crates/studyfrog-service/src/mcp_client_io.rs` (a child process over standard input and output, Streamable HTTP through the application's transport with `Mcp-Session-Id` and server-sent events) and `mcp_servers.rs`; `crates/studyfrog-ui/src/mcp_client_ui.rs`, `mcp_tool_calls.rs`, `view/agent_servers.rs`. `studyfrog-net` now returns the answer's headers.
- Configuration lives in `mcp_servers.toml` in the config folder of the application home (not in the vault): name, `command` + `args` or `url`, `enabled`, `trust_read_only`, `allowed`. New servers start enabled and untrusted. Consent works like the providers': a remote HTTP server gets nothing until "Allow sending text to <host>" is ticked; a local program or localhost address needs no consent.
- Connections tab: "Tool servers (MCP)" lists each server with "Use", "Run read-only tools without asking", the remote consent, "Discover" (sends `initialize`, `tools/list`, `resources/list`, `resources/templates/list`, `prompts/list`; lists a server lacks stay empty) and "Remove", and shows the discovered tools with read-only / changes things / may delete badges, resources and prompts.
- Assistant tasks ("Work the vault") and exams: the usable tools' names, descriptions and argument schemas are added to the model's instructions as text, and the model asks for a call with a small JSON object. This was chosen over each provider's native tool-calling API so it works the same with all five providers; the cost is that a model must follow the instruction. A call runs at once only if the tool has `readOnlyHint: true` and the server is trusted; otherwise the card "Allow Assistant to run tool <name> on server <server>?" shows the JSON arguments with "Allow this call" / "Do not allow". The result, an error or the refusal goes back to the model; at most 4 tool calls per answer. Each discovery or call opens its own short session.
- Build vs buy: written in-house on the existing JSON-RPC types (no MCP client crate); the protocol subset needed is small and the crates pull an async runtime.
- Tests (fake local servers only): a shell-script MCP server run as a subprocess (`crates/studyfrog-service/tests/fixtures/fake_mcp_server.sh`: discovery, calls, log lines skipped, error answers, a timeout), StudyFrog's own router in-process, a fake Streamable HTTP endpoint with sessions and server-sent events, `send` returning headers against a local socket; in the egui harness, registering, discovering and showing a server, and an exam where a changing tool waits for "Allow", a refused call goes back to the model, and a trusted read-only tool runs at once, plus a wiki task using a tool. Never tried with a real external MCP server or a real model. Not built: keeping a server process running between calls, server-to-client requests (sampling, roots), reading external resources or prompts into a conversation.

## Dependencies
EPIC-037, STORY-0117.

## Open Questions
None.
