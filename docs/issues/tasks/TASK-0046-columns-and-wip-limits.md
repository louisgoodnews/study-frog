---
id: TASK-0046
type: task
title: "Columns and WIP limits"
status: done
epic: EPIC-011
parents: [STORY-0021]
estimate: 3
labels: [m5, board]
acceptance_criteria:
  - "`Board`, `Column` (name, optional WIP limit), `Card` as plain data"
  - "Default board: Backlog, Doing (limit 3), Review, Done"
  - "Adding, renaming, reordering and removing columns keeps every card"
  - "Invariants: unique ids, every card in exactly one column"
---

## Context
Kanban: limit work in progress.

## Description
Columns and WIP limits.

## Acceptance Criteria
- [x] `Board`, `Column` (name, optional WIP limit), `Card` as plain data
- [x] Default board: Backlog, Doing (limit 3), Review, Done
- [x] Adding, renaming, reordering and removing columns keeps every card
- [x] Invariants: unique ids, every card in exactly one column

## Design Notes
`studyfrog-board::model` (Board, Column, Card, default board Backlog/Doing(3)/Review/Done) and `edit` (add/update/remove card, add/edit/remove/move column). Removing a column hands its cards to the previous column (the next if it was first), which may leave that column over its limit; lowering a limit below the count flags the column and moves nothing.

## Dependencies
None.

## Open Questions
What happens to cards when their column is removed? Decision: they move to the previous column (or next if first).
