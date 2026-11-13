---
id: TASK-0017
type: task
title: "FSRS property and golden tests"
status: done
epic: EPIC-004
parents: [STORY-0007]
estimate: 3
labels: [m1, srs]
acceptance_criteria:
  - "Golden vectors from the reference implementation"
  - "proptests: interval monotonic in rating, stability non-negative, retrievability in [0,1] and decreasing"
---

## Context
Verification for TASK-0016.

## Description
FSRS property and golden tests.

## Acceptance Criteria
- [x] Golden vectors from the reference implementation
- [x] proptests: interval monotonic in rating, stability non-negative, retrievability in [0,1] and decreasing

## Design Notes
Golden source: py-fsrs `tests/test_basic.py::test_review_card` (intervals `[0,2,11,46,163,498,0,0,2,4,7,12,21]`). The harness emulates learning steps (1m, 10m) as plain data. Reference code was not executed: running the external package was blocked by the sandbox policy, so expected values come from the reference test file.

## Dependencies
TASK-0016.

## Open Questions
None.
