---
id: STORY-0121
type: story
title: Built-in Assistant MCP client host for external academic tool servers
status: backlog
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
- [ ] Users can register external MCP servers (local stdio command or remote Streamable HTTP URL) in the Assistant Connections tab and inspect their discovered tools, resources, and prompts
- [ ] When running an Assistant task or interactive exam, StudyFrog can attach discovered MCP tool schemas to the LLM request and execute approved tool calls
- [ ] External MCP tools without readOnlyHint: true require explicit user confirmation in the Assistant UI before execution

## Design Notes
- Touchpoints:
  - `crates/studyfrog-agent/src/mcp_client.rs`, `crates/studyfrog-service/src/agent.rs`, `crates/studyfrog-ui/src/view/agent.rs`.
- Strictly enforce P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-037, STORY-0117.

## Open Questions
None.
