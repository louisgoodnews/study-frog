---
id: TASK-0201
type: task
title: "Item saved toast when leaving the editor"
status: done
epic: EPIC-026
parents: [STORY-0075]
estimate: 2
labels: [m11, ui]
acceptance_criteria:
  - "Leaving the editor after at least one save shows a toast 'Saved <title>' that goes away by itself after about three seconds"
  - "No toast appears for each auto-save while the user is editing, and none when nothing was saved"
  - "A failed save raises an error notice that stays until dismissed"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
The user wants to see that their work was saved without being told after every word.

## Acceptance Criteria
- [x] Leaving the editor after at least one save shows a toast 'Saved <title>' that goes away by itself after about three seconds
- [x] No toast appears for each auto-save while the user is editing, and none when nothing was saved
- [x] A failed save raises an error notice that stays until dismissed

## Design Notes
Done as `editor_leave.rs` (toast on leaving after at least one save) and `Toast.expires_ms`.

Owner's answer (2026-11-03): a toast on every auto-save is too noisy; perhaps one when leaving the edit view. The editor counts the saves of the visit; leaving (back, another screen, another item, exit) raises the toast. Toasts today stay until dismissed; give `Toast` an optional lifetime that the update drops when `State.now` passes it.

## Dependencies
TASK-0200.

## Open Questions
None.
