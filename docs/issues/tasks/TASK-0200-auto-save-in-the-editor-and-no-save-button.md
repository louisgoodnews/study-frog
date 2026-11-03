---
id: TASK-0200
type: task
title: Auto-save in the editor and no Save button
status: done
epic: EPIC-026
parents: [STORY-0075]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "An edit is saved when a token is finished: the user typed a whitespace or a line break after changed text"
  - "At the latest 5 seconds after the last unsaved edit the text is saved even if no token was finished"
  - "A short pause alone does not save"
  - "The Save button is gone; Revert and Reload from disk stay"
  - "Leaving the editor, opening another item and exiting save first"
  - "A file changed on disk since it was loaded is never overwritten: the editor says so and offers Reload or Keep mine"
  - "Saves do not pile up: at most one is running, and a token finished meanwhile is saved right after"
  - "A failed save is shown and retried, and the unsaved state is visible"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
Saving by hand is easy to forget; the editor should keep the file up to date by itself.

## Acceptance Criteria
- [x] An edit is saved when a token is finished: the user typed a whitespace or a line break after changed text
- [x] At the latest 5 seconds after the last unsaved edit the text is saved even if no token was finished
- [x] A short pause alone does not save
- [x] The Save button is gone; Revert and Reload from disk stay
- [x] Leaving the editor, opening another item and exiting save first
- [x] A file changed on disk since it was loaded is never overwritten: the editor says so and offers Reload or Keep mine
- [x] Saves do not pile up: at most one is running, and a token finished meanwhile is saved right after
- [x] A failed save is shown and retried, and the unsaved state is visible

## Design Notes
Done as `editor_save.rs` (token rule `token_done`, 5 s limit by the Tick, one save at a time with `again`, `flush_before` for leaving/opening/exiting, conflict stops auto-save and offers Reload or Keep mine) and a status text in the toolbar instead of the Save button. The old exit question for unsaved changes no longer shows because exiting saves first.

Owner's answer (2026-11-03): no save after a pause of about a second; only on update, token-wise: read the text until a whitespace and take everything before it as the updated text. The 5 second limit from the first request stays for text without a whitespace. In the pure state: `Editor` keeps the time of the first unsaved edit and the last character typed; `update` emits `SaveDocument` when the last typed character is whitespace or a line break and the form is dirty, or when 5 seconds passed. The existing hash check keeps its conflict rule. `has_unsaved_changes` is true only while a save is pending or failed. Every save changes the file, so check what the vault poll and the incremental reindex cost per word and coalesce if needed.

## Dependencies
None.

## Open Questions
None.
