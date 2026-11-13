---
id: STORY-0117
type: story
title: "Pure JSON-RPC 2.0 MCP protocol engine and extensible plain-data capability registry"
status: done
epic: EPIC-037
parents: [EPIC-037]
estimate: 5
labels: [m12, agent, ext, service]
acceptance_criteria:
  - "StudyFrog implements a pure, I/O-free JSON-RPC 2.0 MCP router supporting initialize, notifications/initialized, ping, tools/list, tools/call, resources/list, resources/templates/list, resources/read, prompts/list, and prompts/get"
  - "McpRegistry stores McpTool, McpResourceTemplate, and McpPrompt definitions as plain data structs with function pointers and MCP ToolAnnotations (readOnlyHint, destructiveHint, idempotentHint, openWorldHint)"
  - "Adding a new MCP tool, resource, or prompt requires only registering a plain data struct entry without modifying the protocol router or using OOP trait objects"
---

## Context
To make StudyFrog's Model Context Protocol (MCP) support genuinely future-proof and aligned with StudyFrog's strict functional architecture (`docs/research/mcp-architecture.md`), the core protocol layer must be 100% decoupled from physical transport (`stdio` vs. Streamable HTTP) and represent capabilities as **plain data structs of function pointers** (the same pattern used in `studyfrog-ext`).

## Description
Build the pure MCP protocol engine and extensible `McpRegistry`:
1. **JSON-RPC 2.0 Framing & Stateless Context (`crates/studyfrog-agent/src/mcp_wire.rs` or `crates/studyfrog-mcp`):**
   - Parse and serialize JSON-RPC 2.0 requests, responses, notifications, and standard error codes (`-32700` Parse error, `-32600` Invalid Request, `-32601` Method not found, `-32602` Invalid params, `-32603` Internal error).
   - Handle `initialize` capability negotiation (advertising `tools`, `resources`, and `prompts` capabilities and protocol version `2025-06-18`) and `ping`.
2. **Plain-Data `McpRegistry` & `McpToolAnnotations`:**
   - Define plain structs:
     - `McpToolAnnotations { title, read_only_hint, destructive_hint, idempotent_hint, open_world_hint }`
     - `McpTool { name, description, input_schema_json, annotations, call: fn(...) -> Result<McpContent, McpError> }`
     - `McpResourceTemplate { uri_template, name, description, mime_type, read: fn(...) -> Result<McpResourceContent, McpError> }`
     - `McpPrompt { name, description, arguments, get: fn(...) -> Result<Vec<McpPromptMessage>, McpError> }`
   - Provide pure `register_tool`, `register_resource`, and `register_prompt` functions so both built-in service modules and external extensions can extend the server registry cleanly.
3. **Safety Gate Filter:**
   - Support an `McpPolicy { allow_write: bool, allow_destructive: bool }` that automatically filters or gates `tools/call` based on each tool's `McpToolAnnotations`.

## Acceptance Criteria
- [x] StudyFrog implements a pure, I/O-free JSON-RPC 2.0 MCP router supporting initialize, notifications/initialized, ping, tools/list, tools/call, resources/list, resources/templates/list, resources/read, prompts/list, and prompts/get
- [x] McpRegistry stores McpTool, McpResourceTemplate, and McpPrompt definitions as plain data structs with function pointers and MCP ToolAnnotations (readOnlyHint, destructiveHint, idempotentHint, openWorldHint)
- [x] Adding a new MCP tool, resource, or prompt requires only registering a plain data struct entry without modifying the protocol router or using OOP trait objects

## Design Notes
- Zero I/O in the protocol router: `handle_mcp_message(registry: &McpRegistry, policy: McpPolicy, session: &mut VaultSession, raw_json: &str) -> Option<String>`.
- Strictly comply with P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-037.

## Open Questions
None.
