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
  - "Familiarity is computed for each (item, stack) with that stack's exam date and settings, and the reports and the Needs attention card of a stack use the stack's value"
  - "The item's overall value is the average over its references and is what the item page, the library column and the item reports show, together with the value per stack"
  - "The values named in the answer to question (b) of TASK-0210 are averaged the same way"
  - "The replay that judges familiarity (TASK-0184) still works and says which values it used"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized as in Obsidian's file explorer; and the schedule, the due date, the familiarity and other values of an item belong to each stack that references it, the item's overall value being the average over its references (Flashcard 100 in stack A with 5 weeks and in stack B with 3 weeks).

## Description
The owner: familiarity and the other values count in reference to the stack the item is viewed by, and the overall value of a flashcard is the average of all its references.

## Acceptance Criteria
- [ ] Familiarity is computed for each (item, stack) with that stack's exam date and settings, and the reports and the Needs attention card of a stack use the stack's value
- [ ] The item's overall value is the average over its references and is what the item page, the library column and the item reports show, together with the value per stack
- [ ] The values named in the answer to question (b) of TASK-0210 are averaged the same way
- [ ] The replay that judges familiarity (TASK-0184) still works and says which values it used

## Design Notes
`studyfrog_srs::familiarity` is pure and takes the item's history and the exam distance; call it per stack with the stack's inputs and keep a small `average_over` function. The library column 'Familiarity' shows the average with the per-stack values in the tooltip.

## Dependencies
TASK-0218.

## Open Questions
None.
