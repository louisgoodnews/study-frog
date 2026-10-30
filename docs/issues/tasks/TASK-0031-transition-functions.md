---
id: TASK-0031
type: task
title: Transition functions
status: done
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
- [x] `step` implements Idle -> Running <-> Paused -> Finished|Aborted
- [x] Events emitted per spec with correct seq numbers
- [x] proptest: any input sequence keeps invariants (seq contiguous, no events after end)

## Design Notes
`step(&mut Session, Input)` validates before mutating, so refusals leave the session unchanged (tested for 10 cases). Start shows the first item; rating or skipping shows the next or finishes. Abort is allowed from Idle. Free-text answers need answer mode; Flip reveals in either mode. Property test: random action sequences keep seq contiguous, at most one terminal event, no input accepted after the end.

## Dependencies
TASK-0030.

## Open Questions
None.
