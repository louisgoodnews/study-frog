---
id: TASK-0213
type: task
title: References in the interface
status: backlog
epic: EPIC-026
parents: [STORY-0079]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "The item page shows the primary stack and the stacks that reference the item, with add and remove"
  - "An item can be added to another stack from the library's bulk bar and by dragging it onto a stack"
  - "A stack's pages mark the items that are only referenced (with their primary stack's name)"
  - "The Create dialog's parent stack stays the primary stack and offers an optional 'Also in' choice"
  - "Every action has an accessible name and a keyboard route"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0. On the same day the owner decided about stacks: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; stacks are not folders per se, but a folder can be marked as a 'stack' folder and is then also treated as a stack, while a 'non-stack' folder is a normal folder.

## Description
How the user puts an item into more than one stack.

## Acceptance Criteria
- [ ] The item page shows the primary stack and the stacks that reference the item, with add and remove
- [ ] An item can be added to another stack from the library's bulk bar and by dragging it onto a stack
- [ ] A stack's pages mark the items that are only referenced (with their primary stack's name)
- [ ] The Create dialog's parent stack stays the primary stack and offers an optional 'Also in' choice
- [ ] Every action has an accessible name and a keyboard route

## Design Notes
Builds on the bulk bar and the stack page's Backlog and List tabs.

## Dependencies
TASK-0212.

## Open Questions
None.
