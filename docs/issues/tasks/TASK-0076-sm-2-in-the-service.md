---
id: TASK-0076
type: task
title: SM-2 in the service
status: done
epic: EPIC-018
parents: [STORY-0034]
estimate: 5
labels: [m3, service, store, srs]
acceptance_criteria:
  - "`record_step` applies the item's stack scheduler (default FSRS); SM-2 state is stored with its due time and last review"
  - "Queue candidates use the item's current scheduler, falling back to the other scheduler's due date after a switch"
  - "`request_retention` override applies to FSRS intervals"
  - "Reviews stay one shared history"
---

## Context
SM-2 is the simple user-selectable mode (docs/specs/srs-algorithm.md).

## Description
SM-2 in the service.

## Acceptance Criteria
- [x] `record_step` applies the item's stack scheduler (default FSRS); SM-2 state is stored with its due time and last review
- [x] Queue candidates use the item's current scheduler, falling back to the other scheduler's due date after a switch
- [x] `request_retention` override applies to FSRS intervals
- [x] Reviews stay one shared history

## Design Notes
Each item uses its stack's scheduler, else the vault default (`studyfrog.toml`). `apply_rating_sm2` is pure; `record_step` writes `sm2_state` or `mem_state` plus the shared `review` row. `load_schedules` returns a unified `Scheduled` (FSRS state kept for difficulty filters). A stack retention override changes FSRS params per item. Tested: SM-2 stack 1 then 6 days, retention 0.8 gives 8 days, vault default vs stack override, scheduler switch.

## Dependencies
TASK-0075.

## Open Questions
Switching a stack's scheduler mid-way: the other scheduler's due date keeps the item scheduled; the next review starts fresh state in the new scheduler.
