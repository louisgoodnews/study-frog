---
id: TASK-0018
type: task
title: "SM-2 functions"
status: done
epic: EPIC-004
parents: [STORY-0008]
estimate: 3
labels: [m1, srs]
acceptance_criteria:
  - "SM-2 next state and interval as pure functions"
  - "proptests for bounds"
---

## Context
Baseline algorithm.

## Description
SM-2 functions.

## Acceptance Criteria
- [x] SM-2 next state and interval as pure functions
- [x] proptests for bounds

## Design Notes
Source: super-memory.com SM-2 page (verified 2026-10-30). Grade mapping Again=1, Hard=3, Good=4, Easy=5 is ours; intervals round up per the original. `Sm2State` lives in core. SM-2 tests are hand-computed from the original rules, not reference-implementation vectors.

## Dependencies
TASK-0014.

## Open Questions
None.
