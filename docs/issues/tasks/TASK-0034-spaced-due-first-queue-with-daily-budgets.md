---
id: TASK-0034
type: task
title: "Spaced (due-first) queue with daily budgets"
status: done
epic: EPIC-007
parents: [STORY-0015]
estimate: 3
labels: [m3, rehearsal]
acceptance_criteria:
  - "Spaced order: due items first (most overdue first), then new items; daily new and review budgets are inputs"
  - "Deterministic given a seed for any shuffling"
  - "proptest: respects limits, no duplicates"
---

## Context
Inputs: due list and new list as plain data.

## Description
Spaced (due-first) queue with daily budgets.

## Acceptance Criteria
- [x] Spaced order: due items first (most overdue first), then new items; daily new and review budgets are inputs
- [x] Deterministic given a seed for any shuffling
- [x] proptest: respects limits, no duplicates

## Design Notes
SoA vs AoS decision for the event log is taken here with data (EPIC-003 open question).

## Dependencies
TASK-0033.

## Open Questions
None.
