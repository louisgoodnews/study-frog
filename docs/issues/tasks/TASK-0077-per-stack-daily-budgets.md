---
id: TASK-0077
type: task
title: Per-stack daily budgets
status: done
epic: EPIC-018
parents: [STORY-0034]
estimate: 5
labels: [m3, service, rehearsal, store]
acceptance_criteria:
  - "Spaced order applies each stack's remaining new/review allowance plus an optional global budget"
  - "Remaining = limit minus today's distinct new and reviewed items, from the review history"
  - "Day boundaries are injected (`DayWindow`), practice-only runs never count"
---

## Context
Anki-style daily limits.

## Description
Per-stack daily budgets.

## Acceptance Criteria
- [x] Spaced order applies each stack's remaining new/review allowance plus an optional global budget
- [x] Remaining = limit minus today's distinct new and reviewed items, from the review history
- [x] Day boundaries are injected (`DayWindow`), practice-only runs never count

## Design Notes
`Budgets { global, per_stack }` in rehearsal: per-stack allowances are applied in queue order (most overdue first), then the global cap. The service computes each stack's remaining allowance as limit minus today's distinct new/reviewed items (`DayWindow` injected). Tested: allowance spent within a day, practice runs do not count, next day resets, review limit.

## Dependencies
TASK-0075.

## Open Questions
None.
