---
id: TASK-0213
type: task
title: References in the interface
status: done
epic: EPIC-026
parents: [STORY-0079]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "The item page shows the primary stack and the stacks that reference the item, with add and remove, and the schedule, due date and familiarity of each"
  - "An item can be added to another stack from the library's bulk bar and by dragging it onto a stack"
  - "A stack's pages mark the items that are only referenced (with their primary stack's name)"
  - "The Create dialog's parent stack stays the primary stack and offers an optional 'Also in' choice"
  - "Every action has an accessible name and a keyboard route"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized as in Obsidian's file explorer; and the schedule, the due date, the familiarity and other values of an item belong to each stack that references it, the item's overall value being the average over its references (Flashcard 100 in stack A with 5 weeks and in stack B with 3 weeks).

## Description
How the user puts an item into more than one stack and sees what each stack makes of it.

## Acceptance Criteria
- [x] The item page shows the primary stack and the stacks that reference the item, with add and remove, and the schedule, due date and familiarity of each
- [x] An item can be added to another stack from the library's bulk bar and by dragging it onto a stack
- [x] A stack's pages mark the items that are only referenced (with their primary stack's name)
- [x] The Create dialog's parent stack stays the primary stack and offers an optional 'Also in' choice
- [x] Every action has an accessible name and a keyboard route

## Design Notes
Builds on the bulk bar and the stack page's Backlog and List tabs.

## Dependencies
TASK-0212 and TASK-0218.

## Open Questions
None.
