---
id: TASK-0016
type: task
title: FSRS pure functions
status: backlog
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
- [ ] next_state, interval_days, retrievability per spec (FSRS-6, 21 weights)
- [ ] No clock or RNG inside; fuzz seed is an argument

## Design Notes
Take remaining equations from the reference implementation.

## Dependencies
TASK-0014.

## Open Questions
None.
