---
id: TASK-0213
type: task
title: Stack membership in the interface
status: backlog
epic: EPIC-026
parents: [STORY-0079]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "A stack's contents can be added to and removed from by a picker and by drag and drop from the Items view"
  - "The item page shows the stacks an item is in"
  - "The Create dialog's 'Parent stack' becomes 'Add to stack' with the same default"
  - "Every action has an accessible name and a keyboard route"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
How the user puts items into stacks once a stack is a file.

## Acceptance Criteria
- [ ] A stack's contents can be added to and removed from by a picker and by drag and drop from the Items view
- [ ] The item page shows the stacks an item is in
- [ ] The Create dialog's 'Parent stack' becomes 'Add to stack' with the same default
- [ ] Every action has an accessible name and a keyboard route

## Design Notes
Builds on the stack page's Backlog and List tabs and the bulk bar of the library.

## Dependencies
TASK-0212.

## Open Questions
None.
