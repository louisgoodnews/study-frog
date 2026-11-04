---
id: STORY-0079
type: story
title: Stack folders and items referenced by several stacks
status: done
epic: EPIC-026
parents: [EPIC-026]
estimate: 8
labels: [m11, store, md, srs, ui, decision]
acceptance_criteria:
  - "A folder is a stack when it is marked as one and is a normal folder otherwise, and the user can change that"
  - "An item has one primary stack (where it lives) and can be referenced by other stacks through its `referenced_by` field"
  - "Each stack that references an item has its own schedule, due date and familiarity for it, and the item's overall value is the average over its references"
  - "Queues, reports, the calendar and import and export follow the membership"
  - "Existing vaults keep working: the old schedule becomes the primary stack's"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized as in Obsidian's file explorer; and the schedule, the due date, the familiarity and other values of an item belong to each stack that references it, the item's overall value being the average over its references (Flashcard 100 in stack A with 5 weeks and in stack B with 3 weeks).

## Description
Today a folder with a `_stack.md` is a stack, its items are the files inside it, and each item has one schedule. The owner wants an item usable in several stacks, each with its own schedule and values, and folders that are only folders. The stack file is already the mark of a stack folder, so old vaults stay valid; the per-stack schedule is the larger change because reviews and schedules are per item today.

## Acceptance Criteria
- [x] A folder is a stack when it is marked as one and is a normal folder otherwise, and the user can change that
- [x] An item has one primary stack (where it lives) and can be referenced by other stacks through its `referenced_by` field
- [x] Each stack that references an item has its own schedule, due date and familiarity for it, and the item's overall value is the average over its references
- [x] Queues, reports, the calendar and import and export follow the membership
- [x] Existing vaults keep working: the old schedule becomes the primary stack's

## Design Notes
Tasks TASK-0210 to TASK-0213, TASK-0217, TASK-0218, TASK-0219 and TASK-0220. The decision record comes first; the schedule change (TASK-0218) is the riskiest part: a schema migration, the scheduler and the queues.

## Dependencies
None.

## Open Questions
None.
