---
id: TASK-0201
type: task
title: Item saved toast
status: backlog
epic: EPIC-026
parents: [STORY-0075]
estimate: 2
labels: [m11, ui]
acceptance_criteria:
  - "A toast says that the item was saved after every save, replaces the one before it and goes away by itself after about two seconds"
  - "It does not add an entry to the notification list for every auto-save"
  - "A failed save raises an error notice that stays"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
The user wants to see that their work was saved.

## Acceptance Criteria
- [ ] A toast says that the item was saved after every save, replaces the one before it and goes away by itself after about two seconds
- [ ] It does not add an entry to the notification list for every auto-save
- [ ] A failed save raises an error notice that stays

## Design Notes
Toasts today stay until dismissed; add an optional lifetime to `Toast` and let the update drop it when `State.now` passes it. Whether saves go to the list at all is open.

## Dependencies
TASK-0200.

## Open Questions
One toast for every auto-save may be noisy while typing; should the first save of a session show it and the rest only a small 'Saved' mark in the editor's header?
