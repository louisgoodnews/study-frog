---
id: TASK-0047
type: task
title: "Move and push semantics"
status: done
epic: EPIC-011
parents: [STORY-0021]
estimate: 5
labels: [m5, board]
acceptance_criteria:
  - "`move_card` to a column and position; Strict refuses a move that would exceed a limit, Force allows it"
  - "Reordering inside a column is always allowed"
  - "`push` moves a card one column forward, `pull` one back, to the top of the target"
  - "Property tests: cards are conserved and no limit is exceeded by Strict moves from a valid board"
---

## Context
The pull-system rule of kanban.

## Description
Move and push semantics.

## Acceptance Criteria
- [x] `move_card` to a column and position; Strict refuses a move that would exceed a limit, Force allows it
- [x] Reordering inside a column is always allowed
- [x] `push` moves a card one column forward, `pull` one back, to the top of the target
- [x] Property tests: cards are conserved and no limit is exceeded by Strict moves from a valid board

## Design Notes
`moves`: `move_card`, `push`, `pull`, `shift`; `ops::BoardOp` + `apply` as the command form. Strict refuses entering a full column, Force allows it and the column shows as over limit; reordering inside a column is always allowed. Property tests: 80-operation random sequences keep ids unique, card count exact and never exceed a limit in Strict mode; move then reverse restores the board exactly.

## Dependencies
TASK-0046.

## Open Questions
None.
