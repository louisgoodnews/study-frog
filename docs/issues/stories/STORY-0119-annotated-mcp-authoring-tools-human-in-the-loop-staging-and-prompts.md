---
id: STORY-0119
type: story
title: Annotated MCP authoring tools, human-in-the-loop staging, and parameterized MCP prompts
status: backlog
epic: EPIC-037
parents: [EPIC-037]
estimate: 5
labels: [m12, agent, service, ui]
acceptance_criteria:
  - "All built-in StudyFrog academic prompts are exposed via MCP prompts/list and prompts/get with typed arguments (file_path, target_stack, depth, count)"
  - "Additive MCP authoring tools (vault_propose_items, note_extract_inline_cards, diary_log_entry, calendar_plan_session) validate inputs through studyfrog-service and support human-in-the-loop staging by default"
  - "Destructive or structural tools (vault_move_stack, vault_delete_item) carry destructiveHint: true and require explicit write/destructive policy approval and record full undo entries"
---

## Context
External AI agents connected via MCP should be able to help learners generate flashcards, extract notes, log diary reflections, and organize stacks—while strictly preserving StudyFrog's human-in-the-loop safety model (`ADR 0009`: external models never silently overwrite or delete user vault files without explicit permission or staging). In addition, exposing StudyFrog's built-in prompt library as native MCP Prompts (`prompts/list`, `prompts/get`) allows any MCP client to invoke them via `/` slash commands.

## Description
Implement MCP Prompts and Tier-2/Tier-3 Annotated Authoring Tools:
1. **Native MCP Prompts (`prompts/list`, `prompts/get`):**
   - Map `studyfrog_service::prompts::PROMPTS` (`flashcards`, `questions`, `stack`, `notes`, `summary`, `critique`, and planned `explain`, `exam`, `socratic`, `concept_map`) into `McpPrompt` entries with optional arguments (`file_path` to auto-attach a vault file up to `ATTACH_LIMIT`, `target_stack`, and prompt-specific parameters).
2. **Tier-2 Additive / Staged Authoring Tools (`readOnlyHint: false`, `destructiveHint: false`):**
   - `vault_propose_items`: Accepts a batch of cards/questions/notes (JSON or StudyFrog CSV), validates and cleans them via `studyfrog-service` (including newline unescaping and full titles), and either stages them into the Assistant approval queue (default) or writes them directly when `--mcp-allow-write` is active.
   - ` diary_log_entry`: Records a Learning Diary reflection with 1–5 self-assessment ratings.
   - `calendar_plan_session`: Creates a planned study session on the vault calendar.
3. **Tier-3 Structural / Destructive Tools (`destructiveHint: true`):**
   - `vault_move_stack`: Moves a stack between a parent stack and standalone top-level while preserving all item schedules (`STORY-0100`).
   - `vault_delete_item`: Deletes an item and records a full undo snapshot (`STORY-0101`), gated by `policy.allow_destructive`.

## Acceptance Criteria
- [ ] All built-in StudyFrog academic prompts are exposed via MCP prompts/list and prompts/get with typed arguments (file_path, target_stack, depth, count)
- [ ] Additive MCP authoring tools (vault_propose_items, note_extract_inline_cards, diary_log_entry, calendar_plan_session) validate inputs through studyfrog-service and support human-in-the-loop staging by default
- [ ] Destructive or structural tools (vault_move_stack, vault_delete_item) carry destructiveHint: true and require explicit write/destructive policy approval and record full undo entries

## Design Notes
- Touchpoints:
  - `crates/studyfrog-service/src/mcp_prompts.rs` and `mcp_tools_write.rs`.
- Strictly enforce P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-037, STORY-0117, STORY-0118.

## Open Questions
None.
