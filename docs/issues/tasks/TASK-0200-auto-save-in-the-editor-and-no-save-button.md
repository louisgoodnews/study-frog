---
id: TASK-0200
type: task
title: Auto-save in the editor and no Save button
status: backlog
epic: EPIC-026
parents: [STORY-0075]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "An edit is saved when the user pauses typing and at the latest 5 seconds after the last edit"
  - "The Save button is gone; Revert and Reload from disk stay"
  - "Leaving the editor, opening another item and exiting save first"
  - "A file changed on disk since it was loaded is never overwritten: the editor says so and offers Reload or Keep mine"
  - "A failed save is shown and retried, and the unsaved state is visible"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
Saving by hand is easy to forget; the editor should keep the file up to date by itself.

## Acceptance Criteria
- [ ] An edit is saved when the user pauses typing and at the latest 5 seconds after the last edit
- [ ] The Save button is gone; Revert and Reload from disk stay
- [ ] Leaving the editor, opening another item and exiting save first
- [ ] A file changed on disk since it was loaded is never overwritten: the editor says so and offers Reload or Keep mine
- [ ] A failed save is shown and retried, and the unsaved state is visible

## Design Notes
A debounce on the pure state: `Editor` gets the time of the last edit (from `State.now`), and the tick or the next frame emits `SaveDocument` when the form is dirty and the pause or the 5 second limit has passed. The existing hash check keeps its conflict rule. `has_unsaved_changes` becomes true only while a save is pending or has failed.

## Dependencies
None.

## Open Questions
Should a pause of about one second also save (a shorter pause than the 5 seconds)? The owner wrote 'on edit or latest 5 seconds after'.
