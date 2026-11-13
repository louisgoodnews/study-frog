---
id: TASK-0016
type: task
title: "FSRS pure functions"
status: done
epic: EPIC-004
parents: [STORY-0007]
estimate: 5
labels: [m1, srs]
acceptance_criteria:
  - "next_state, interval_days, retrievability per spec (FSRS-6, 21 weights)"
  - "No clock or RNG inside; fuzz seed is an argument"
---

## Context
Spec pins FSRS-6.

## Description
FSRS pure functions.

## Acceptance Criteria
- [x] next_state, interval_days, retrievability per spec (FSRS-6, 21 weights)
- [x] No clock or RNG inside; fuzz seed is an argument

## Design Notes
Equations follow the FSRS-6 wiki and the py-fsrs scheduler (cross-checked: the golden test reproduces its intervals). `MemState` lives in `core` because the store persists it. Elapsed days are whole days; `< 1` selects the short-term path. f32 throughout. Weights are not range-validated yet (optimiser work, post-M7).

## Dependencies
TASK-0014.

## Open Questions
None.
