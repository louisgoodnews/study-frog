---
id: TASK-0080
type: task
title: Persist the board
status: done
epic: EPIC-011
parents: [STORY-0021]
estimate: 3
labels: [m5, store, service]
acceptance_criteria:
  - "Schema v4 stores columns and cards with their order"
  - "Save and load round-trip exactly; one transaction per change"
  - "Service use cases apply a pure board operation and store the result"
---

## Context
ADR 0003: board data is authoritative in SQLite.

## Description
Persist the board.

## Acceptance Criteria
- [x] Schema v4 stores columns and cards with their order
- [x] Save and load round-trip exactly; one transaction per change
- [x] Service use cases apply a pure board operation and store the result

## Design Notes
Schema v4 `board_column` + `board_card` (cascade); `store::board_store::{save_board, load_board}` replace the whole board in one transaction; service `kanban::apply_board_op` loads (creating the default board once), applies a `BoardOp`, stores, and turns refusals into readable messages. Restart round-trip tested.

## Dependencies
TASK-0046.

## Open Questions
None.
