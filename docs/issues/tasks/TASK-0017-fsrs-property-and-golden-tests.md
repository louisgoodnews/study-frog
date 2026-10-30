---
id: TASK-0017
type: task
title: FSRS property and golden tests
status: backlog
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
- [ ] Golden vectors from the reference implementation
- [ ] proptests: interval monotonic in rating, stability non-negative, retrievability in [0,1] and decreasing

## Design Notes
proptest regressions committed.

## Dependencies
TASK-0016.

## Open Questions
None.
