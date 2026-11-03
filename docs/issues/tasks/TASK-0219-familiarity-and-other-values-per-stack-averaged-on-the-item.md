---
id: TASK-0219
type: task
title: Familiarity and other values per stack, averaged on the item
status: backlog
epic: EPIC-026
parents: [STORY-0079]
estimate: 5
labels: [m11, srs, report, ui]
acceptance_criteria:
  - "Familiarity is computed for each (item, stack) from the reviews made in that stack, with that stack's exam date and settings, and the reports and the Needs attention card of a stack use the stack's value"
  - "The item's overall value is the average over its references; the item page and the library show the average and the value per stack in the owner's shape (`source`, `rating`, `familiarity`, with due date)"
  - "The JSON export carries the per-stack list and the average"
  - "The replay that judges familiarity (TASK-0184) still works and says which values it used"
  - "Where the values are stored follows the answer to question (1) of TASK-0210"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized like Obsidian's file explorer; each stack that references an item has its own memory of it (schedule, due date, familiarity), independent of the others, so rating a card in stack A does not touch stack B; the item's overall value is the average over its references; in a run over several stacks a card is due if it is due in any of them and a rating updates every selected stack that holds the card, never a stack outside the selection.

## Description
The owner: familiarity and the other values count in reference to the stack the item is viewed by, and the overall value of a flashcard is the average of all its references.

## Acceptance Criteria
- [ ] Familiarity is computed for each (item, stack) from the reviews made in that stack, with that stack's exam date and settings, and the reports and the Needs attention card of a stack use the stack's value
- [ ] The item's overall value is the average over its references; the item page and the library show the average and the value per stack in the owner's shape (`source`, `rating`, `familiarity`, with due date)
- [ ] The JSON export carries the per-stack list and the average
- [ ] The replay that judges familiarity (TASK-0184) still works and says which values it used
- [ ] Where the values are stored follows the answer to question (1) of TASK-0210

## Design Notes
`studyfrog_srs::familiarity` is pure and takes the item's history and the exam distance; call it per stack with that stack's reviews and inputs and keep a small `average_over` function. The library column 'Familiarity' shows the average with the per-stack values in the tooltip.

## Dependencies
TASK-0218.

## Open Questions
None.
