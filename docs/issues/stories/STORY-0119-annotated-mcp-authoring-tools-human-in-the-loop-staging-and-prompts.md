---
id: STORY-0119
type: story
title: "Annotated MCP authoring tools, human-in-the-loop staging, and parameterized MCP prompts"
status: done
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
- [x] All built-in StudyFrog academic prompts are exposed via MCP prompts/list and prompts/get with typed arguments (file_path, target_stack, depth, count)
- [x] Additive MCP authoring tools (vault_propose_items, note_extract_inline_cards, diary_log_entry, calendar_plan_session) validate inputs through studyfrog-service and support human-in-the-loop staging by default
- [x] Destructive or structural tools (vault_move_stack, vault_delete_item) carry destructiveHint: true and require explicit write/destructive policy approval and record full undo entries

## Design Notes
- Touchpoints:
  - `crates/studyfrog-service/src/mcp_prompts.rs` and `mcp_tools_write.rs`.
- Strictly enforce P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Done. Built in `crates/studyfrog-ui/src/mcp_write.rs`, `mcp_extract.rs`, `mcp_structural.rs`, `mcp_prompts.rs`, `mcp_ui.rs`, `exec_mcp.rs`, `view/agent_staged.rs`; `crates/studyfrog-agent/src/mcp_stage.rs` and `wiki_change.rs`; `crates/studyfrog-service/src/mcp_staging.rs`, `mcp_undo.rs`, `agent_apply_more.rs`; `crates/studyfrog-md/src/inline_extract.rs`. (The touchpoints named above were in the service crate; the MCP context lives in the UI crate, as for STORY-0118.)
- Prompts: every prompt of `PROMPTS` is an MCP prompt with `file_path` (a vault file, clipped to `ATTACH_LIMIT`, paths outside the vault refused) and `target_stack`, plus `depth` (`explain`) and `count` (`exam`). Tested in `crates/studyfrog-ui/tests/mcp_structural.rs`.
- Staging: tools carry a `staged` flag; the policy lists and runs them without `--mcp-allow-write`, and then they only write a JSON proposal to `.studyfrog/staged/` in the vault. `vault_propose_items` (items or StudyFrog CSV, `\n` written out, stacks by folder, id or title), `note_extract_inline_cards` (`Term :: Definition`, `Q:`/`A:`, `??:`/`A:`, code skipped, existing cards skipped), `diary_log_entry` (1 to 5 ratings) and `calendar_plan_session` check their input and stage; with `--mcp-allow-write` they apply directly through `apply_changes`. The Assistant's "Work the vault" tab lists staged proposals; "Review" opens them in the usual review card, "Apply" writes the accepted changes and removes the file, "Discard" removes it. Tested in `mcp_authoring.rs`, `mcp_staged_ui.rs` (harness: review, accept one of two, apply, discard) and `crates/studyfrog-agent/tests/mcp_stage.rs`.
- Structural: `vault_move_stack` and `vault_delete_item` are `destructiveHint: true`, listed and run only with both `--mcp-allow-write` and `--mcp-allow-destructive`, and keep a full undo entry (files, contents, dropped schedules) in `.studyfrog/mcp-undo/`, undone from "Changes by AI tools". Tested: refused under weaker policies, the deleted file comes back byte for byte with its schedule, a move is undone after reopening the vault.
- The inline parser is a first step of STORY-0110 (no sync, no backlinks).

## Dependencies
EPIC-037, STORY-0117, STORY-0118.

## Open Questions
None.
