---
id: STORY-0079
type: story
title: Stacks as files that point to their contents
status: backlog
epic: EPIC-026
parents: [EPIC-026]
estimate: 8
labels: [m11, store, md, decision]
acceptance_criteria:
  - "Folders are only folders; a stack is a file that points to its contents"
  - "Existing vaults keep working and can be migrated"
  - "Queues, reports, the calendar and import and export follow the new membership"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
Today a stack is a folder with a `_stack.md` and its items are the files inside it. The owner wants folders to be pure organization and a stack to be a file that points to its contents and also carries the logic (scheduler, limits, goals, due date, status, subjects). This changes the data model, so it starts with a decision.

## Acceptance Criteria
- [ ] Folders are only folders; a stack is a file that points to its contents
- [ ] Existing vaults keep working and can be migrated
- [ ] Queues, reports, the calendar and import and export follow the new membership

## Design Notes
Tasks TASK-0210 to TASK-0213. This is the largest change of 0.4.0: the owner should decide after the decision record whether the implementation belongs in 0.4.0 or 0.5.0.

## Dependencies
None.

## Open Questions
None.
