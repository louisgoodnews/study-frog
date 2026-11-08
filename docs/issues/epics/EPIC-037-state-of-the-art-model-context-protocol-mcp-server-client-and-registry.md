---
id: EPIC-037
type: epic
title: State-of-the-art Model Context Protocol (MCP) server, client host, and extensible registry
status: done
parents: []
labels: [m12, agent, ext, service, ui, app]
acceptance_criteria:
  - "StudyFrog includes a pure, transport-agnostic JSON-RPC 2.0 MCP engine and a plain-data registry (McpRegistry) for registering Tools (with safety annotations), Resource templates, and Prompts without OOP traits"
  - "Running studyfrog-app --mcp exposes the vault over stdio or localhost Streamable HTTP with canonical studyfrog:// resources, read-only search/analytics/forecast tools, staged authoring tools, and parameterized prompts"
  - "The built-in Assistant can act as an MCP Host/Client connecting to external MCP servers (such as literature search, citation managers, or local domain tools) with explicit user permission gates"
---

## Context
The **Model Context Protocol (MCP)** is the open industry standard (JSON-RPC 2.0) for connecting AI assistants, IDEs, and autonomous agents with structured application resources, executable tools, and reusable prompt templates.

As detailed in `docs/research/mcp-architecture.md`, StudyFrog needs a state-of-the-art, future-proof MCP architecture that:
1. Allows external AI tools (Claude Desktop, Cursor, Zed, Cline, custom research agents) to inspect StudyFrog vaults (`studyfrog://` resources), run learning analytics/forecasts, use StudyFrog's importer-tested academic prompts, and propose new flashcards/notes safely via `studyfrog-service` rather than corrupting raw `.md` or `.sqlite` files.
2. Allows StudyFrog's built-in Assistant to connect to external MCP tool servers (e.g., Zotero, PubMed/arXiv, local Python/Rust servers).
3. Remains 100% faithful to StudyFrog's functional, data-oriented architecture: **plain-data function-pointer registries** (like `studyfrog-ext`), **zero inherent `impl` blocks (P1)**, **≤ 250 lines per file (P2)**, **strict crate layering (P3)**, and **`unsafe_code = "forbid"` (P9)**.

## Description
Implement a complete, extensible Model Context Protocol (MCP) stack across StudyFrog:
1. **Pure MCP Protocol Engine & Plain-Data Registry (`STORY-0117`):**
   - Implement JSON-RPC 2.0 message framing, capability negotiation (`initialize`, `ping`, stateless request context), error codes, and `McpRegistry` holding plain-data `McpTool` (with `ToolAnnotations`: `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`), `McpResourceTemplate`, and `McpPrompt` descriptors.
2. **MCP Server Resources (`studyfrog://`) & Read-Only Analytics/Search Tools (`STORY-0118`):**
   - Expose `studyfrog://vault/summary`, `studyfrog://stacks/{stack}`, `studyfrog://items/{id_or_key}`, `studyfrog://reports/{kind}`, and `studyfrog://diary/{date}` resources, plus read-only tools (`vault_search_items`, `vault_get_due_queue`, `analytics_get_report`, `forecast_simulate_workload`, `pdf_read_pages`).
3. **Annotated MCP Authoring Tools, Human-in-the-Loop Staging & Prompts (`STORY-0119`):**
   - Expose additive/staged authoring tools (`vault_propose_items`, `note_extract_inline_cards`, `diary_log_entry`, `calendar_plan_session`), gated structural tools (`vault_move_stack`, `vault_delete_item`), and all built-in StudyFrog prompts via `prompts/list` and `prompts/get`.
4. **CLI `stdio` Server (`studyfrog-app --mcp`) & Localhost Streamable HTTP Transport (`STORY-0120`):**
   - Add `--mcp [--vault <path>] [--mcp-allow-write]` to `studyfrog-app` for standard `stdin`/`stdout` MCP connections, plus an optional localhost-bound (`127.0.0.1`) Streamable HTTP endpoint with bearer token and `Origin` header validation.
5. **Built-In Assistant MCP Client Host (`STORY-0121`):**
   - Allow users to configure external MCP servers (`stdio` subprocess or HTTP URL) in the Assistant Connections tab, discover their tools/resources, and invoke them during Assistant tasks with explicit per-tool approval.

## Acceptance Criteria
- [x] StudyFrog includes a pure, transport-agnostic JSON-RPC 2.0 MCP engine and a plain-data registry (McpRegistry) for registering Tools (with safety annotations), Resource templates, and Prompts without OOP traits
- [x] Running studyfrog-app --mcp exposes the vault over stdio or localhost Streamable HTTP with canonical studyfrog:// resources, read-only search/analytics/forecast tools, staged authoring tools, and parameterized prompts
- [x] The built-in Assistant can act as an MCP Host/Client connecting to external MCP servers (such as literature search, citation managers, or local domain tools) with explicit user permission gates

## Design Notes
Child stories:
- STORY-0117: Pure JSON-RPC 2.0 MCP protocol engine and extensible plain-data capability registry (estimate: 5)
- STORY-0118: StudyFrog MCP server resources and read-only search, analytics, and forecast tools (estimate: 5)
- STORY-0119: Annotated MCP authoring tools, human-in-the-loop staging, and parameterized MCP prompts (estimate: 5)
- STORY-0120: CLI stdio MCP server mode and localhost Streamable HTTP transport (estimate: 5)
- STORY-0121: Built-in Assistant MCP client host for external academic tool servers (estimate: 8)

Reference architecture: `docs/research/mcp-architecture.md`.

## Progress (2026-11-08)
STORY-0117, STORY-0119, STORY-0120 and STORY-0121 are done; STORY-0118 is in review because leech diagnostics, the study-minute forecast and true retention curves wait for EPIC-034. The epic stays in review until STORY-0118 is done. Nothing was tried with a real AI client or a real external MCP server.

## Dependencies
EPIC-016 (Plugin Boundaries), EPIC-021 (Knowledge Workspace), and EPIC-028 (Extensibility).

## Open Questions
None.
