---
id: TASK-0199
type: task
title: Open the new item from the notification that says it was created
status: done
epic: EPIC-026
parents: [STORY-0075]
estimate: 3
labels: [m11, ui]
acceptance_criteria:
  - "The notice after Create says what was made and has an Open button that opens the new item in the editor"
  - "The notification list keeps the link, so it still works later"
  - "Create another keeps the dialog open and still raises the notice for each item"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
After creating an item the user has to find it in the library to edit it.

## Acceptance Criteria
- [x] The notice after Create says what was made and has an Open button that opens the new item in the editor
- [x] The notification list keeps the link, so it still works later
- [x] Create another keeps the dialog open and still raises the notice for each item

## Design Notes
Done as: `Created.uid`, `State.notice_link` (text and uid; the status bar shows Open while that notice is showing) and the notification's `NotifyTarget::Item`. No separate toast: the dialog's notice already is the status-bar notice, and a toast would repeat it.

Add a `ToastAction::OpenItem(uid)` and an optional target on a notification record (`notify_types.rs`), set by the Created handler from the new item's uid. Clicking the list entry navigates like the toast does.

## Dependencies
None.

## Open Questions
None.
