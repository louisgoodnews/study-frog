---
id: TASK-0031
type: task
title: Transition functions
status: backlog
epic: EPIC-007
parents: [STORY-0013]
estimate: 5
labels: [m3, rehearsal]
acceptance_criteria:
  - "`step` implements Idle -> Running <-> Paused -> Finished|Aborted"
  - "Events emitted per spec with correct seq numbers"
  - "proptest: any input sequence keeps invariants (seq contiguous, no events after end)"
---

## Context
Pure functions over RunState; no clock.

## Description
Transition functions.

## Acceptance Criteria
- [ ] `step` implements Idle -> Running <-> Paused -> Finished|Aborted
- [ ] Events emitted per spec with correct seq numbers
- [ ] proptest: any input sequence keeps invariants (seq contiguous, no events after end)

## Design Notes
Clock reading arrives as a field of the input.

## Dependencies
TASK-0030.

## Open Questions
None.
